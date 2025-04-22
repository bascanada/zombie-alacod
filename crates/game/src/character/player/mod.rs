pub mod control;
pub mod jjrs;
pub mod input;
pub mod config;

use bevy::prelude::*;
use ggrs::PlayerHandle;

#[derive(Component, Reflect, Default, Debug, Copy, Clone)]
#[reflect(Component)]
pub struct Player {
   pub handle: PlayerHandle,
}