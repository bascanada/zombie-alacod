mod args;

use animation::{toggle_layer, ActiveLayers, AnimationState, FacingDirection};
use args::get_args;
use bevy::{asset::AssetMetaCheck, prelude::*, utils::hashbrown::HashMap, window::WindowResolution};
use game::{character::{movement::Velocity, player::{config::{PlayerConfig, PlayerConfigHandles}, control::{get_input_map, PlayerAction}, LocalPlayer, Player}}, collider::{spawn_test_wall, CollisionSettings}, frame::FrameDebugUIPlugin, jjrs::{GggrsConnectionConfiguration, GggrsSessionConfiguration}, plugins::{AppState, BaseZombieGamePlugin}};

use utils::{web::WebPlugin};


fn character_equipment_system(
    mut commands: Commands,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<(Entity, &mut ActiveLayers)>,
) {
    for (entity, mut active_layers) in query.iter_mut() {
        // Toggle helmet layer when 'H' is pressed
        if keyboard_input.just_pressed(KeyCode::KeyH) {
            toggle_layer(
                entity,
                &mut commands,
                &mut active_layers,
                vec!["hair".to_string()],
            );
        }
    }
}


fn main() {
    
    let (local_port,mut nbr_player, players, _, matchbox, lobby) = get_args();

    if nbr_player == 0 { nbr_player = players.len() }

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
                .set(AssetPlugin {
                    meta_check: AssetMetaCheck::Never,
                    ..Default::default()
                })
                .set(window_plugin),
        )
        .add_plugins(WebPlugin{})
        .add_plugins(FrameDebugUIPlugin)
        .add_plugins(BaseZombieGamePlugin::new(matchbox != ""))
        .insert_resource(GggrsSessionConfiguration { matchbox: matchbox != "", lobby: lobby.clone(), matchbox_url: matchbox.clone(), connection: GggrsConnectionConfiguration { input_delay: 5, max_player: nbr_player, desync_interval: 10, socket: players.len() > 1, udp_port: local_port}, players: players })
        .add_systems(OnEnter(AppState::InGame), setup)
        .add_systems(Update, character_equipment_system)
        .run();
}

fn setup(mut commands: Commands, collision_settings: Res<CollisionSettings>) {
    spawn_test_wall(
        &mut commands,
        Vec3::new(500.0, 250.0, 0.0),
        Vec2::new(125.0, 500.0),
        &collision_settings,
        Color::rgb(0.6, 0.3, 0.3), // Reddish color
    );
    spawn_test_wall(
        &mut commands,
        Vec3::new(-500.0, 250.0, 0.0),
        Vec2::new(125.0, 500.0),
        &collision_settings,
        Color::rgb(0.6, 0.3, 0.3), // Reddish color
    );

}
