use animation::{
    AnimationBundle, AnimationMapConfig, AnimationState, CharacterAnimationHandles,
    D2AnimationPlugin, SpriteSheetConfig,
};

use leafwing_input_manager::{plugin::InputManagerPlugin, prelude::*};
use bevy::{prelude::*, utils::hashbrown::HashMap, window::WindowResolution};
use game::{character::{movement::Velocity, player::{config::{PlayerConfig, PlayerConfigHandles}, control::{get_input_map, PlayerAction}, Player}}, plugins::BaseZombieGamePlugin};

use bevy_ggrs::{
    AddRollbackCommandExtension, GgrsConfig, LocalInputs, LocalPlayers, PlayerInputs, Rollback,
    Session,
};

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
        .add_plugins(WebPlugin {})
        .add_plugins(BaseZombieGamePlugin)
        .add_systems(Startup, (setup_camera, setup))
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    const PLAYER_SPRITESHEET_CONFIG_PATH: &str = "ZombieShooter/Sprites/Character/player_sheet.ron";
    const PLAYER_ANIMATIONS_CONFIG_PATH: &str =
        "ZombieShooter/Sprites/Character/player_animation.ron";
    const PLAYER_CONFIG_PATH: &str = "ZombieShooter/Sprites/Character/player_config.ron";

    let sprite_sheet_handle: Handle<SpriteSheetConfig> =
        asset_server.load(PLAYER_SPRITESHEET_CONFIG_PATH);
    let animation_handle: Handle<AnimationMapConfig> =
        asset_server.load(PLAYER_ANIMATIONS_CONFIG_PATH);
    let player_config_handle: Handle<PlayerConfig> = asset_server.load(PLAYER_CONFIG_PATH);

    //let mut accessories = HashMap::new();
    //accessories.insert("SHIRT_1".into(), sprint_sheet_shirt_1_handle.clone());

    let animation_bundle =
        AnimationBundle::new(sprite_sheet_handle.clone(), animation_handle.clone());

    commands.spawn((
        Transform::from_scale(Vec3::splat(6.0)).with_translation(Vec3::new(-50.0, 0.0, 0.0)),

        Player { handle: 0 },
        Velocity(Vec2::ZERO),

        InputManagerBundle::<PlayerAction> {
            action_state: ActionState::default(),
            input_map: get_input_map(),
        },

        PlayerConfigHandles {
            config: player_config_handle,
        },

        animation_bundle,
    )).add_rollback();
}
