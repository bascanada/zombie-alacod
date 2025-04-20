use bevy::{prelude::*, window::WindowResolution};

use game::plugins::BaseZombieGamePlugin;
use map::{
    ldtk::{
        plugins::MyWorldInspectorPlugin,
    },
};
use utils::{camera::tod::setup_camera, web::WebPlugin};

use std::env;

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
        .add_plugins(MyWorldInspectorPlugin)
        .add_systems(Startup, setup_camera)
        .add_plugins(WebPlugin {})
        .add_plugins(BaseZombieGamePlugin)
        .run();
}