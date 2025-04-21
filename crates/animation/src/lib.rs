use bevy::{prelude::*, reflect::TypePath, utils::HashMap};
use bevy_common_assets::ron::RonAssetPlugin;
use serde::Deserialize;


// CONFIG

// -- Sprite Sheet Layout Configuration --
#[derive(Asset, TypePath, Deserialize, Debug, Clone)]
pub struct SpriteSheetConfig {
    pub path: String,
    pub tile_size: (u32, u32),
    pub columns: u32,
    pub rows: u32,
}

// -- Animation Definition Configuration --
#[derive(Deserialize, Debug, Clone)]
pub struct AnimationIndices {
    pub start: usize,
    pub end: usize, // Inclusive end index
}

#[derive(Asset, TypePath, Deserialize, Debug, Clone)]
pub struct AnimationMapConfig {
    pub frame_duration: u64,
    pub animations: HashMap<String, AnimationIndices>,
}



// COMPONENT

#[derive(Component, Reflect, Default, Clone, Debug, PartialEq, Eq)]
#[reflect(Component, PartialEq)] // Reflect needed for GGRS state hashing
pub struct AnimationState(pub String);


// Handles are loaded once, assume they don't change and don't need rollback/reflection
#[derive(Component)]
pub struct CharacterAnimationHandles {
    pub spritesheet: Handle<SpriteSheetConfig>,
    pub animations: Handle<AnimationMapConfig>
}

#[derive(Component)]
struct AnimationTimer {
    frame_timer: Timer
}


// Bundle

#[derive(Bundle)]
pub struct AnimationBundle {
    state: AnimationState,
    handles: CharacterAnimationHandles,
    timer: AnimationTimer,
}

impl AnimationBundle {
    pub fn new(
        spritesheet: Handle<SpriteSheetConfig>,
        animations: Handle<AnimationMapConfig>,
    ) -> Self {
        Self {
            state: AnimationState("Idle".into()),
            timer: AnimationTimer{ frame_timer: Timer::from_seconds(1., TimerMode::Repeating)},
            handles: CharacterAnimationHandles {
                spritesheet,
                animations,
            }
        }
    }
}


// Animates sprite based on AnimationState (visual, non-deterministic part) (mostly unchanged)
// Needs to query With<Rollback>
fn animate_sprite_system(
    time: Res<Time>, // Use Bevy's normal time for visual animation speed
    animation_configs: Res<Assets<AnimationMapConfig>>,
    mut query: Query<(&CharacterAnimationHandles, &mut AnimationTimer, &mut Sprite, &AnimationState)>
) {
     for (config_handles, mut timer, mut sprite, state) in query.iter_mut() {
        if let Some(anim_config) = animation_configs.get(&config_handles.animations) {
            timer.frame_timer.tick(time.delta());
            if timer.frame_timer.just_finished() {
                if let Some(atlas) = &mut sprite.texture_atlas {
                    if let Some(indices) = anim_config.animations.get(&state.0) {
                        let start_index = indices.start;
                        let end_index = indices.end;
                        if atlas.index < start_index || atlas.index > end_index {
                            atlas.index = start_index;
                        } else {
                            atlas.index = (atlas.index + 1 - start_index) % (end_index - start_index + 1) + start_index;
                        }
                } else {
                        atlas.index = anim_config.animations.get("Idle").map_or(0, |idx| idx.start);
                    }
                }
            }
        }
    }
}


// Updates animation timer duration if AnimationMapConfig reloads (mostly unchanged)
// Needs to query With<Rollback>
fn check_animation_config_reload_system(
    mut ev_asset: EventReader<AssetEvent<AnimationMapConfig>>,
    animation_configs: Res<Assets<AnimationMapConfig>>,
    mut query: Query<(&CharacterAnimationHandles, &mut AnimationTimer)>,
    asset_server: Res<AssetServer>,
) {
     let mut updates_needed = HashMap::new(); // Handle ID -> new duration

     // Collect updates needed from asset events
     for event in ev_asset.read() {
         match event {
             AssetEvent::Added { id } | AssetEvent::Modified { id } => {
                 if let Some(config) = animation_configs.get(*id) {
                     updates_needed.insert(*id, config.frame_duration);
                 }
             }
             _ => {}
         }
     }

     // Apply updates to relevant entities
     for (config_handles, mut anim_timer) in query.iter_mut() {
         if let Some(new_duration) = updates_needed.get(&config_handles.animations.id()) {
             anim_timer.frame_timer.set_duration(bevy::utils::Duration::from_millis(*new_duration));
             anim_timer.frame_timer.reset();
         }
         // Apply initial duration after startup load (if needed)
         else if anim_timer.frame_timer.duration().as_secs_f32() == 0.1 { // Check default
             if asset_server.load_state(&config_handles.animations).is_loaded() {
                  if let Some(config) = animation_configs.get(&config_handles.animations) {
                     anim_timer.frame_timer.set_duration(bevy::utils::Duration::from_millis(config.frame_duration));
                     anim_timer.frame_timer.reset();
                 }
             }
         }
     }
}


fn character_visuals_update_system(
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    spritesheet_configs: Res<Assets<SpriteSheetConfig>>,
    mut ev_asset: EventReader<AssetEvent<SpriteSheetConfig>>,
    // Query for the TextureAtlasSprite component to modify its index
    mut query: Query<
        (Entity, &CharacterAnimationHandles, &mut Sprite), // <-- Query TextureAtlasSprite
    >,
) {
     for event in ev_asset.read() {
        if let AssetEvent::Modified { id } = event {
            // Find entities using the modified spritesheet config
            for (entity, config_handles, mut sprite) in query.iter_mut() { // <-- sprite is TextureAtlasSprite
                if config_handles.spritesheet.id() == *id {
                    if let Some(new_config) = spritesheet_configs.get(&config_handles.spritesheet) {
                        info!("Spritesheet config modified for GGRS entity {:?}, updating visuals.", entity);
                        let new_layout = TextureAtlasLayout::from_grid(UVec2::new(new_config.tile_size.0, new_config.tile_size.1), new_config.columns, new_config.rows, None, None);

                        sprite.texture_atlas = Some(TextureAtlas { layout: texture_atlas_layouts.add(new_layout), index: 0 });
                        sprite.image = asset_server.load(&new_config.path);

                    }
                }
            }
        }
    }
}


fn character_visuals_spawn_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    spritesheet_configs: Res<Assets<SpriteSheetConfig>>,
    query: Query<(Entity, &CharacterAnimationHandles), Without<Sprite>>,
) {
    for (entity, config_handles) in query.iter() {
        if let Some(spritesheet_config) = spritesheet_configs.get(&config_handles.spritesheet) {
            if asset_server.load_state(&config_handles.spritesheet).is_loaded() {
                let texture_handle: Handle<Image> = asset_server.load(&spritesheet_config.path);
                let layout = TextureAtlasLayout::from_grid(
                    UVec2::new(spritesheet_config.tile_size.0, spritesheet_config.tile_size.1),// spritesheet_config.tile_size,
                    spritesheet_config.columns,
                    spritesheet_config.rows,
                    None, None,
                );
                let layout_handle = texture_atlas_layouts.add(layout);

                commands.entity(entity).insert(
                Sprite {
                            image: texture_handle.clone(),
                            texture_atlas: Some(TextureAtlas {
                                layout: layout_handle.clone(),
                                index: 0
                            }),
                        ..default()
                    },
                );
            }
        }
    }
}


// PLUGIN

pub struct D2AnimationPlugin;

impl Plugin for D2AnimationPlugin {
    fn build(&self, app: &mut App) {

        app.add_plugins(RonAssetPlugin::<SpriteSheetConfig>::new(&["ron"]));
        app.add_plugins(RonAssetPlugin::<AnimationMapConfig>::new(&["ron"]));


        app.add_systems(Update, (
            character_visuals_spawn_system,
            character_visuals_update_system.after(character_visuals_spawn_system),

            animate_sprite_system,
            check_animation_config_reload_system.after(animate_sprite_system),
        ));
    }
}