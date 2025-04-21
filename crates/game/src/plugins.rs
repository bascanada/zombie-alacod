use bevy::prelude::*;

use animation::D2AnimationPlugin;

pub struct BaseZombieGamePlugin;

impl Plugin for BaseZombieGamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(D2AnimationPlugin);
    }
}
