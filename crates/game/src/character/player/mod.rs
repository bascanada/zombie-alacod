pub mod control;
pub mod jjrs;

use bevy::prelude::*;
use ggrs::PlayerHandle;

#[derive(Component, Reflect, Default, Debug, Copy, Clone)]
#[reflect(Component)]
pub struct Player {
   pub handle: PlayerHandle,
}