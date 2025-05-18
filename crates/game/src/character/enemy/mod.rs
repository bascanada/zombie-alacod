pub mod create;
pub mod spawning;
pub mod spawning_basic;


use bevy::prelude::*;


#[derive(Component, Reflect, Default, Debug)]
#[reflect(Component)]
pub struct Enemy {
    
}