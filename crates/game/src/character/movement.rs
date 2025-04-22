use bevy::{prelude::*, reflect::TypePath};
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct MovementConfig {
    pub acceleration: f32,
    pub max_speed: f32,
    pub friction: f32,
}


#[derive(Component, Default, Reflect, Deref, DerefMut, Clone)]
pub struct Velocity(pub Vec2);
