use bevy::{prelude::*, reflect::TypePath};
use serde::Deserialize;

use crate::character::movement::MovementConfig;

#[derive(Asset, TypePath, Deserialize, Debug, Clone)]
pub struct PlayerConfig {
    pub movement: MovementConfig,
}

#[derive(Component)]
pub struct PlayerConfigHandles {
    pub config: Handle<PlayerConfig>
}
