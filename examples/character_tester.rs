use animation::{
    AnimationBundle, AnimationMapConfig, AnimationState, CharacterAnimationHandles,
    D2AnimationPlugin, SpriteSheetConfig,
};
use bevy::{prelude::*, utils::hashbrown::HashMap, window::WindowResolution};
use rand::seq::SliceRandom;

use utils::{camera::tod::setup_camera, web::WebPlugin};

fn main() {
    let window_plugin = WindowPlugin {
        primary_window: Some(Window {
            title: "zrl-character_tester".to_string(),
            resolution: WindowResolution::new(800., 600.),

            resizable: true,
            #[cfg(target_arch = "wasm32")]
            canvas: Some("#bevy-canvas".to_string()),
            ..Default::default()
        }),
        ..Default::default()
    };

    App::new()
        .add_plugins(
            DefaultPlugins
                .set(ImagePlugin::default_nearest())
                .set(window_plugin),
        )
        .add_plugins(D2AnimationPlugin)
        .add_systems(Startup, (setup_camera, setup))
        .add_systems(Update, random_animation_on_r_key_system)
        .add_plugins(WebPlugin {})
        .run();
}

fn random_animation_on_r_key_system(
    keyboard_input: Res<ButtonInput<KeyCode>>, // Access keyboard state
    animation_configs: Res<Assets<AnimationMapConfig>>, // Access loaded animation maps
    mut query: Query<(&mut AnimationState, &CharacterAnimationHandles)>,
) {
    if keyboard_input.just_pressed(KeyCode::KeyR) {
        let mut rng = rand::thread_rng(); // Create a random number generator

        for (mut current_state, anim_map_handle) in query.iter_mut() {
            if let Some(anim_config) = animation_configs.get(&anim_map_handle.animations) {
                // Adjust .0 if using direct Handle

                let all_animation_names: Vec<String> =
                    anim_config.animations.keys().cloned().collect();

                let possible_new_states: Vec<&String> = all_animation_names
                    .iter()
                    .filter(|&name| *name != current_state.0) // Ensure new state is different
                    .collect();

                if let Some(new_state_name) = possible_new_states.choose(&mut rng) {
                    info!(
                        "Changing animation state from '{}' to '{}'",
                        current_state.0, new_state_name
                    );
                    current_state.0 = (*new_state_name).clone(); // Update the state component
                }
            }
        }
    }
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    const PLAYER_SPRITESHEET_CONFIG_PATH: &str = "ZombieShooter/Sprites/Character/player_sheet.ron";
    const PLAYER_ANIMATIONS_CONFIG_PATH: &str =
        "ZombieShooter/Sprites/Character/player_animation.ron";

    let sprite_sheet_handle: Handle<SpriteSheetConfig> =
        asset_server.load(PLAYER_SPRITESHEET_CONFIG_PATH);
    let animation_handle: Handle<AnimationMapConfig> =
        asset_server.load(PLAYER_ANIMATIONS_CONFIG_PATH);

    //let mut accessories = HashMap::new();
    //accessories.insert("SHIRT_1".into(), sprint_sheet_shirt_1_handle.clone());

    let animation_bundle =
        AnimationBundle::new(sprite_sheet_handle.clone(), animation_handle.clone());

    commands.spawn((
        Transform::from_scale(Vec3::splat(6.0)).with_translation(Vec3::new(-50.0, 0.0, 0.0)),
        animation_bundle,
    ));
}
