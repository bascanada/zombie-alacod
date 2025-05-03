use bevy::{prelude::*, reflect::TypePath, utils::HashMap};
use serde::Deserialize;

use crate::character::movement::MovementConfig;




#[derive(Asset, TypePath, Deserialize, Debug, Clone)]
pub struct PlayerConfig {
    pub movement: MovementConfig,
    pub default_skin: String,
    pub skins: HashMap<String, Vec<String>>
}

#[derive(Component)]
pub struct PlayerConfigHandles {
    pub config: Handle<PlayerConfig>
}
