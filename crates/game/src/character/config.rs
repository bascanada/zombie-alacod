use bevy::{prelude::*, reflect::TypePath, utils::HashMap};
use serde::Deserialize;

use crate::character::movement::MovementConfig;


#[derive(Asset, TypePath, Deserialize, Debug, Clone)]
pub struct CharacterConfig {
    pub movement: MovementConfig,
}

#[derive(Component)]
pub struct CharacterConfigHandles {
    pub config: Handle<CharacterConfig>
}
