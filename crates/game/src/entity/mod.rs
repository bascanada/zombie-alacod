use bevy::prelude::*;

pub mod character;
pub mod player;


pub struct CharacterPlugin {}

impl Plugin for CharacterPlugin {
    fn build(&self, app: &mut App) {}
    
}