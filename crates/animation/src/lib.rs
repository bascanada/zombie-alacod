use bevy::{prelude::*, reflect::TypePath, sprite::Anchor, utils::HashMap};
use bevy_ggrs::{prelude::*, GgrsSchedule};
use bevy_common_assets::ron::RonAssetPlugin;
use serde::Deserialize;

// CONFIG

// 1a. Define your custom enum that CAN be deserialized
#[derive(Deserialize, Debug, Clone, Copy)]
#[serde(rename_all = "PascalCase")] // Allows "BottomCenter" in RON file
pub enum ConfigurableAnchor {
    Center,
    BottomLeft,
    BottomCenter,
    BottomRight,
    CenterLeft,
    CenterRight,
    TopLeft,
    TopCenter,
    TopRight,
    // Add Custom(Vec2) if you need it, requires slightly more complex mapping
}

impl ConfigurableAnchor {

    pub fn to_anchor(&self) -> Anchor {
         return match self {
             ConfigurableAnchor::Center => Anchor::Center,
             ConfigurableAnchor::BottomLeft => Anchor::BottomLeft,
             ConfigurableAnchor::BottomCenter => Anchor::BottomCenter,
             ConfigurableAnchor::BottomRight => Anchor::BottomRight,
             ConfigurableAnchor::CenterLeft => Anchor::CenterLeft,
             ConfigurableAnchor::CenterRight => Anchor::CenterRight,
             ConfigurableAnchor::TopLeft => Anchor::TopLeft,
             ConfigurableAnchor::TopCenter => Anchor::TopCenter,
             ConfigurableAnchor::TopRight => Anchor::TopRight,
             // Add Custom case here if you defined it
         };
    }
}

// -- Sprite Sheet Layout Configuration --
#[derive(Asset, TypePath, Deserialize, Debug, Clone)]
pub struct SpriteSheetConfig {
    pub path: String,
    pub tile_size: (u32, u32),
    pub columns: u32,
    pub rows: u32,
    pub name: String,
    pub scale: f32,
    pub offset_x: f32,
    pub offset_y: f32,
    pub offset_z: f32,
    pub animated: bool,
    pub anchor: ConfigurableAnchor,
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
pub struct LoadingAsset {
    pub layers: HashMap<String, String>,
    pub remove: Vec<String>,
}

#[derive(Component)]
pub struct LayerName {
    pub name: String
}

#[derive(Component)]
pub struct AnimatedLayer {}

#[derive(Component)]
pub struct ColoredLayer {}

#[derive(Component, Clone)]
pub struct ActiveLayers {
    pub layers: HashMap<String, String>,
}

#[derive(Component, Reflect, Default, Clone, Debug, PartialEq, Eq)]
#[reflect(Component, PartialEq)] // Reflect needed for GGRS state hashing
pub struct AnimationState(pub String);

// Handles are loaded once, assume they don't change and don't need rollback/reflection
#[derive(Component)]
pub struct CharacterAnimationHandles {
    pub spritesheets: HashMap<String, Handle<SpriteSheetConfig>>,
    pub animations: Handle<AnimationMapConfig>,
}

#[derive(Component)]
struct AnimationTimer {
    frame_timer: Timer,
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum FacingDirection {
    Left,
    Right,
}

impl Default for FacingDirection {
    fn default() -> Self {
        FacingDirection::Right
    }
}

// Bundle

#[derive(Bundle)]
pub struct AnimationBundle {
    state: AnimationState,
    handles: CharacterAnimationHandles,
    timer: AnimationTimer,
    active_layers: ActiveLayers,
    loading_asset: LoadingAsset,
    facing_direction: FacingDirection,
}

impl AnimationBundle {
    pub fn new(
        spritesheets: HashMap<String, Handle<SpriteSheetConfig>>,
        animations: Handle<AnimationMapConfig>,

        starting_layers: HashMap<String, String>,
    ) -> Self {
        Self {
            state: AnimationState("Idle".into()),
            timer: AnimationTimer {
                frame_timer: Timer::from_seconds(1., TimerMode::Repeating),
            },
            handles: CharacterAnimationHandles {
                spritesheets,
                animations,
            },
            loading_asset: LoadingAsset {
                layers: starting_layers,
                remove: vec![],
            },
            active_layers: ActiveLayers {
                layers: HashMap::new(),
            },
            facing_direction: FacingDirection::default()
        }
    }
}






// Animates sprite based on AnimationState
fn animate_sprite_system(
    time: Res<Time>,
    animation_configs: Res<Assets<AnimationMapConfig>>,
    mut query: Query<(
        &Children,
        &CharacterAnimationHandles,
        &mut AnimationTimer,
        &AnimationState,
    ), Without<LoadingAsset>>,
    mut query_sprites: Query<(&mut Sprite, &LayerName), With<AnimatedLayer>>,
    
) {
    for (childs, config_handles, mut timer, state) in query.iter_mut() {
        if let Some(anim_config) = animation_configs.get(&config_handles.animations) {
            timer.frame_timer.tick(time.delta());
            if timer.frame_timer.just_finished() {
                for child in childs.iter() {
                    if let Ok((mut sprite, _)) = query_sprites.get_mut(*child) {
                        if let Some(atlas) = &mut sprite.texture_atlas {
                            if let Some(indices) = anim_config.animations.get(&state.0) {
                                let start_index = indices.start;
                                let end_index = indices.end;
                                if atlas.index < start_index || atlas.index > end_index {
                                    atlas.index = start_index;
                                } else {
                                    atlas.index = (atlas.index + 1 - start_index)
                                        % (end_index - start_index + 1)
                                        + start_index;
                                }
                            } else {
                                atlas.index = anim_config
                                    .animations
                                    .get("Idle")
                                    .map_or(0, |idx| idx.start);
                            }
                        }
                    }
                }
            }
        }
    }
}

// Updates animation timer duration if AnimationMapConfig reloads
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
            anim_timer
                .frame_timer
                .set_duration(bevy::utils::Duration::from_millis(*new_duration));
            anim_timer.frame_timer.reset();
        }
        // Apply initial duration after startup load (if needed)
        else if anim_timer.frame_timer.duration().as_secs_f32() == 1.0 {
            // Check default
            if asset_server
                .load_state(&config_handles.animations)
                .is_loaded()
            {
                if let Some(config) = animation_configs.get(&config_handles.animations) {
                    anim_timer
                        .frame_timer
                        .set_duration(bevy::utils::Duration::from_millis(config.frame_duration));
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
    query: Query<(&Children, Entity, &CharacterAnimationHandles)>,
    mut query_sprite: Query<(&mut Sprite,&mut Transform, &LayerName)>,
) {
    for event in ev_asset.read() {
        if let AssetEvent::Modified { id } = event {
            // Find entities using the modified spritesheet config
            for (childs, entity, config_handle) in query.iter() {
                for handle in config_handle.spritesheets.values() {
                    if handle.id() == *id {
                        if let Some(new_config) = spritesheet_configs.get(handle) {
                            info!(
                                "Spritesheet config modified for entity {:?}, updating visuals.",
                                entity
                            );
                            let new_layout = TextureAtlasLayout::from_grid(
                                UVec2::new(new_config.tile_size.0, new_config.tile_size.1),
                                new_config.columns,
                                new_config.rows,
                                None,
                                None,
                            );

                            for child in childs.iter() {
                                if let Ok((mut sprite, mut transform, layer_name)) = query_sprite.get_mut(*child) {
                                    if layer_name.name == new_config.name {
                                        sprite.texture_atlas = Some(TextureAtlas {
                                            layout: texture_atlas_layouts.add(new_layout.clone()),
                                            index: 0,
                                        });
                                        transform.translation.x = new_config.offset_x;
                                        transform.translation.z = new_config.offset_z;
                                        transform.translation.y = new_config.offset_y;
                                        transform.scale = Vec3::splat(new_config.scale);
                                        sprite.image = asset_server.load(&new_config.path);
                                        sprite.anchor = new_config.anchor.to_anchor();
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}


// SYSTEM THAT RUN ON THE BEVY SCHEDULE FOR SYNCH

pub fn character_visuals_spawn_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    spritesheet_configs: Res<Assets<SpriteSheetConfig>>,
    animation_configs: Res<Assets<AnimationMapConfig>>,
    mut query: Query<(Entity, &CharacterAnimationHandles, &mut ActiveLayers, &mut LoadingAsset, &AnimationState), With<Rollback>>,
    child_query: Query<&Children>,
    sprite_query: Query<(Entity, &LayerName, &Sprite)>,
) {
    for (entity, config_handles, mut active_layers, mut loading_assets, anim_state) in query.iter_mut() {
        let total_items = loading_assets.layers.len() + loading_assets.remove.len();
        let mut processed_count = 0;
        let mut loaded_items = vec![];


        let mut current_frame_index = 0;
        
        // Try to determine the current animation frame from existing sprites
        if let Ok(children) = child_query.get(entity) {
            for child in children.iter() {
                if let Ok((_, _, sprite)) = sprite_query.get(*child) {
                    if let Some(atlas) = &sprite.texture_atlas {
                        current_frame_index = atlas.index;
                        break;
                    }
                }
            }
        }
        
        // If we couldn't find existing sprites, try to determine from animation config
        if current_frame_index == 0 && anim_state.0 != "Idle" {
            if let Some(anim_config) = animation_configs.get(&config_handles.animations) {
                if let Some(indices) = anim_config.animations.get(&anim_state.0) {
                    current_frame_index = indices.start;
                }
            }
        }

        // Handle layer additions
        for spritesheet in config_handles.spritesheets.values() {
            if let Some(spritesheet_config) = spritesheet_configs.get(spritesheet) {
                if let Some(_) = loading_assets.layers.get(&spritesheet_config.name) {
                    if asset_server.load_state(spritesheet).is_loaded() {
                        let texture_handle: Handle<Image> = asset_server.load(&spritesheet_config.path);
                        let layout = TextureAtlasLayout::from_grid(
                            UVec2::new(
                                spritesheet_config.tile_size.0,
                                spritesheet_config.tile_size.1,
                            ),
                            spritesheet_config.columns,
                            spritesheet_config.rows,
                            None,
                            None,
                        );
                        let layout_handle = texture_atlas_layouts.add(layout);

                        let mut entity_commands = commands.spawn((
                            Sprite {
                                image: texture_handle.clone(),
                                texture_atlas: Some(TextureAtlas {
                                    layout: layout_handle.clone(),
                                    index: current_frame_index,
                                }),
                                anchor: spritesheet_config.anchor.to_anchor(),
                                ..default()
                            },
                            Transform::from_scale(Vec3::splat(spritesheet_config.scale))
                                .with_translation(Vec3::new(spritesheet_config.offset_x, spritesheet_config.offset_y, spritesheet_config.offset_z)),
                                //.with_rotation(Quat::IDENTITY),
                            LayerName { name: spritesheet_config.name.clone() },
                        ));

                        if spritesheet_config.animated {
                            entity_commands.insert(AnimatedLayer{});
                        } else {
                            println!("adding non animation layer {}", spritesheet_config.path);
                        }

                        let sprite = entity_commands.id();

                        commands.entity(entity).add_child(sprite);

                        // Add to active layers with empty string value (or you could store meaningful metadata here)
                        active_layers.layers.insert(spritesheet_config.name.clone(), String::new());
                        
                        loaded_items.push(spritesheet_config.name.clone());
                        processed_count += 1;
                        info!("Spawned layer: {}", spritesheet_config.name);
                    }
                }
            }
        }

        // Remove processed items from loading queue
        for key in loaded_items {
            loading_assets.layers.remove(&key);
        }

        // Handle layer removals
        if !loading_assets.remove.is_empty() {
            if let Ok(children) = child_query.get(entity) {
                let mut to_despawn = Vec::new();
                
                for child in children.iter() {
                    if let Ok((child_entity, layer_name, _)) = sprite_query.get(*child) {
                        if loading_assets.remove.contains(&layer_name.name) {
                            to_despawn.push(child_entity);
                            active_layers.layers.remove(&layer_name.name);
                            processed_count += 1;
                            info!("Despawned layer: {}", layer_name.name);
                        }
                    }
                }
                
                // Despawn in a separate loop to avoid borrowing issues
                for child_entity in to_despawn {
                    commands.entity(child_entity).despawn_recursive();
                }
            }
            
            // Clear the removal list
            loading_assets.remove.clear();
        }

        // Remove the LoadingAsset component if all operations are complete
        if processed_count == total_items {
            info!("All layer operations completed. Processed {} items.", processed_count);
            commands.entity(entity).remove::<LoadingAsset>();
        }
    }
}

pub fn set_sprite_flip(
    query: Query<(&Children, &FacingDirection), With<Rollback>>,
    mut sprite_query: Query<(&mut Sprite)>,
) {
    for (childrens, direction) in query.iter() {
        for child in childrens.iter() {
            if let Ok(mut sprite) = sprite_query.get_mut(*child) {
                match direction {
                    FacingDirection::Left => {
                        sprite.flip_x = true;
                    }
                    FacingDirection::Right => {
                        sprite.flip_x = false;
                    }
                }
            }
        }
    }
}

/// Toggles the specified layers on or off for the given entity.
/// If a layer is currently active, it will be removed.
/// If a layer is not active, it will be added.
pub fn toggle_layer(
    parent_entity: Entity,
    commands: &mut Commands,
    active_layers: &mut ActiveLayers,
    layers: Vec<String>,
) {
    let mut to_remove = vec![];
    let mut to_insert = HashMap::new();

    for layer in layers.iter() {
        if active_layers.layers.contains_key(layer) {
            to_remove.push(layer.clone());
        } else {
            to_insert.insert(layer.clone(), String::new());
        }
    }

    // Only proceed if there are actual changes to make
    if !to_remove.is_empty() || !to_insert.is_empty() {
        info!(
            "Layer toggle operation: adding {:?}, removing {:?}",
            to_insert.keys().collect::<Vec<_>>(),
            to_remove
        );
        
        commands.entity(parent_entity).insert(LoadingAsset {
            layers: to_insert,
            remove: to_remove,
        });
    }
}

/// Adds the specified layers to the entity if they aren't already active
pub fn add_layers(
    parent_entity: Entity,
    commands: &mut Commands,
    active_layers: &ActiveLayers,
    layers: Vec<String>,
) {
    let mut to_insert = HashMap::new();

    for layer in layers.iter() {
        if !active_layers.layers.contains_key(layer) {
            to_insert.insert(layer.clone(), String::new());
        }
    }

    if !to_insert.is_empty() {
        info!("Adding layers: {:?}", to_insert.keys().collect::<Vec<_>>());
        commands.entity(parent_entity).insert(LoadingAsset {
            layers: to_insert,
            remove: vec![],
        });
    }
}

/// Removes the specified layers from the entity if they are active
pub fn remove_layers(
    parent_entity: Entity,
    commands: &mut Commands,
    active_layers: &ActiveLayers,
    layers: Vec<String>,
) {
    let mut to_remove = vec![];

    for layer in layers.iter() {
        if active_layers.layers.contains_key(layer) {
            to_remove.push(layer.clone());
        }
    }

    if !to_remove.is_empty() {
        info!("Removing layers: {:?}", to_remove);
        commands.entity(parent_entity).insert(LoadingAsset {
            layers: HashMap::new(),
            remove: to_remove,
        });
    }
}

/// Replaces one layer with another
pub fn replace_layer(
    parent_entity: Entity,
    commands: &mut Commands,
    active_layers: &ActiveLayers,
    old_layer: String,
    new_layer: String,
) {
    let mut to_remove = vec![];
    let mut to_insert = HashMap::new();

    if active_layers.layers.contains_key(&old_layer) {
        to_remove.push(old_layer);
    }
    
    if !active_layers.layers.contains_key(&new_layer) {
        to_insert.insert(new_layer, String::new());
    }

    if !to_remove.is_empty() || !to_insert.is_empty() {
        info!(
            "Replacing layer: {:?} with {:?}",
            to_remove,
            to_insert.keys().collect::<Vec<_>>()
        );
        
        commands.entity(parent_entity).insert(LoadingAsset {
            layers: to_insert,
            remove: to_remove,
        });
    }
}

// PLUGIN

pub struct D2AnimationPlugin;

impl Plugin for D2AnimationPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(RonAssetPlugin::<SpriteSheetConfig>::new(&["ron"]));
        app.add_plugins(RonAssetPlugin::<AnimationMapConfig>::new(&["ron"]));
        
        app
            .rollback_component_with_reflect::<AnimationState>()
            .rollback_component_with_clone::<ActiveLayers>();

        app.add_systems(
            Update,
            (
                character_visuals_update_system,
                animate_sprite_system.after(character_visuals_update_system),
                check_animation_config_reload_system.after(animate_sprite_system),
            )
        );
    }
}