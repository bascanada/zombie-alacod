use bevy::{prelude::*, reflect::TypePath};
use movement::MovementConfig;
use serde::Deserialize;

pub mod movement;
pub mod player;

#[derive(Asset, TypePath, Deserialize, Debug, Clone)]
pub struct PlayerConfig {
    pub movement: MovementConfig,
}

#[derive(Component)]
pub struct PlayerConfigHandles {
    pub config: Handle<PlayerConfig>
}
