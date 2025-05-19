use bevy::prelude::*;


#[derive(Component, Reflect, Default, Clone)]
#[reflect(Component)]
pub struct DashState {
    pub is_dashing: bool,
    pub dash_direction: Vec2,
    pub dash_frames_remaining: u32,
    pub dash_cooldown_remaining: u32,
    pub dash_distance_per_frame: f32,  // Distance to move each frame
    pub dash_start_position: Vec3,     // Starting position for the dash
    pub dash_total_distance: f32,      // Total distance for current dash
}

impl DashState {
    pub fn can_dash(&self) -> bool {
        !self.is_dashing && self.dash_cooldown_remaining == 0
    }
    
    pub fn start_dash(&mut self, direction: Vec2, start_position: Vec3, total_distance: f32, duration_frames: u32) {
        self.is_dashing = true;
        self.dash_direction = direction.normalize();
        self.dash_frames_remaining = duration_frames;
        self.dash_start_position = start_position;
        self.dash_total_distance = total_distance;
        self.dash_distance_per_frame = total_distance / duration_frames as f32;
    }
    
    pub fn update(&mut self) {
        if self.is_dashing {
            self.dash_frames_remaining = self.dash_frames_remaining.saturating_sub(1);
            if self.dash_frames_remaining == 0 {
                self.is_dashing = false;
            }
        }
        
        if self.dash_cooldown_remaining > 0 {
            self.dash_cooldown_remaining = self.dash_cooldown_remaining.saturating_sub(1);
        }
    }
    
    pub fn set_cooldown(&mut self, cooldown_frames: u32) {
        self.dash_cooldown_remaining = cooldown_frames;
    }
}