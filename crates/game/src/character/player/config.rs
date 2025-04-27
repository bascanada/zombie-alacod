use bevy::{prelude::*, reflect::TypePath};
use serde::Deserialize;

use crate::character::movement::MovementConfig;

#[derive(Asset, TypePath, Deserialize, Debug, Clone)]
pub struct PlayerConfig {
    pub movement: MovementConfig,
    pub default_layers: Vec<String>
}

#[derive(Component)]
pub struct PlayerConfigHandles {
    pub config: Handle<PlayerConfig>
}
