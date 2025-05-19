pub mod config;
pub mod movement;
pub mod enemy;
pub mod player;
pub mod health;
pub mod create;
pub mod dash;


use bevy::prelude::*;


#[derive(Component, Clone, Copy, Default)]
pub struct Character;