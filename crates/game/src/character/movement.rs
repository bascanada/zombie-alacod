use animation::AnimationState;
use bevy::{prelude::*};
use bevy_ggrs::Rollback;
use serde::{Deserialize, Serialize};

use super::{config::{CharacterConfig, CharacterConfigHandles}, Character};

#[derive(Deserialize, Debug, Clone)]
pub struct MovementConfig {
    pub acceleration: f32,
    pub max_speed: f32,
    pub friction: f32,
    pub sprint_multiplier: f32,         // How much faster sprint is (e.g. 2.0 for double speed)
    pub sprint_acceleration_per_frame: f32, // How much sprint increases each frame (0-1)
    pub sprint_deceleration_per_frame: f32,

    pub dash_distance: f32,         // Total distance to dash
    pub dash_duration_frames: u32,  // How many frames the dash takes
    pub dash_cooldown_frames: u32,  // Frames before dash can be used again
}

#[derive(Component, Reflect, Default, Clone)]
#[reflect(Component)]
pub struct SprintState {
    pub is_sprinting: bool,
    pub sprint_factor: f32,  // Ranges from 0.0 to 1.0 for gradual acceleration
}


#[derive(Component, Default, Clone, Debug, Serialize, Deserialize)]
pub struct FrameMovementIntent {
    pub delta_position: Vec2,
    pub is_dashing_this_frame: bool, // Flag to signal dash movement
    pub dash_target_position: Option<Vec3>, // If dashing, what's the target for this frame
}