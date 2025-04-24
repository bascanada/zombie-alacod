mod args;

use args::get_args;
use bevy::{prelude::*, utils::hashbrown::HashMap, window::WindowResolution};
use game::{character::{movement::Velocity, player::{config::{PlayerConfig, PlayerConfigHandles}, control::{get_input_map, PlayerAction}, LocalPlayer, Player}}, frame::FrameDebugUIPlugin, jjrs::{GggrsConnectionConfiguration, GggrsSessionConfiguration}, plugins::BaseZombieGamePlugin};

use utils::{camera::tod::setup_camera, web::WebPlugin};



fn main() {
    
    let (local_port,mut nbr_player, players, _, matchbox) = get_args();

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
                .set(window_plugin),
        )
        .add_plugins(WebPlugin{})
        .add_plugins(FrameDebugUIPlugin)
        .add_plugins(BaseZombieGamePlugin::new(matchbox != ""))
        .insert_resource(GggrsSessionConfiguration { matchbox: matchbox != "", matchbox_url: matchbox.clone(), connection: GggrsConnectionConfiguration { input_delay: 5, max_player: nbr_player, desync_interval: 10, socket: players.len() > 1, udp_port: local_port}, players: players })
        .add_systems(Startup, setup_camera)
        .run();
}
