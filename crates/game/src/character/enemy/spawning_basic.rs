use bevy::prelude::*;
use map::game::entity::map::wall::Wall;
use serde::Deserialize;
use utils::math::round_vec3;
use crate::frame::FrameCount;
use utils::rng::RollbackRng;

use super::spawning::{EnemySpawnState, EnemySpawnSystem};
use super::Enemy;


// Implementation for spawning in a circle around players

#[derive(Asset, TypePath, Deserialize, Debug, Clone)]
pub struct BasicEnemySpawnConfig {
    pub timeout_range: (f32, f32),  // Range of seconds between spawns
    pub max_enemies: u32,           // Maximum number of enemies alive at once
    pub spawn_radius: f32,          // Radius around players where zombies can spawn
    pub min_player_distance: f32,   // Minimum distance from any player
}

impl Default for BasicEnemySpawnConfig {
    fn default() -> Self {
        Self {
            timeout_range: (5.0,10.0),
            max_enemies: 5,
            spawn_radius: 1500.0,
            min_player_distance: 800.0,
        }
    }
}

pub struct BasicEnemySpawnSystem {
    config: BasicEnemySpawnConfig,
}


impl BasicEnemySpawnSystem {
    pub fn new(config: BasicEnemySpawnConfig) -> Self {
        Self { config }
    }
}


impl EnemySpawnSystem for BasicEnemySpawnSystem {
    fn should_spawn(&self, frame: &FrameCount, state: &EnemySpawnState, current_enemies: u32) -> bool {
        current_enemies < self.config.max_enemies && (frame.frame >= state.next_spawn_frame)
    }
    
    fn calculate_next_spawn_frame(&self, frame: &FrameCount, rng: &mut RollbackRng) -> u32 {
        let min_frames = (self.config.timeout_range.0 * 60.0) as u32;
        let max_frames = (self.config.timeout_range.1 * 60.0) as u32;
        let frame_delay = min_frames + (rng.next_f32() * (max_frames - min_frames) as f32) as u32;
        frame.frame + frame_delay
    }

    fn calculate_spawn_position(
        &self,
        rng: &mut RollbackRng,
        player_positions: &[Vec2],
        enemy_query: &Query<(Entity, &Transform), With<Enemy>>,
        wall_query: &Query<(Entity, &Transform), With<Wall>>,
    ) -> Option<Vec2> {
        if player_positions.is_empty() {
            return None;
        }
        
        // Calculate center of all players
        let mut center = Vec2::ZERO;
        for pos in player_positions {
            center += *pos;
        }
        center /= player_positions.len() as f32;
        
        // Find radius that encloses all players + spawn radius
        let mut enclosing_radius = self.config.spawn_radius;
        for pos in player_positions {
            let dist = center.distance(*pos);
            enclosing_radius = enclosing_radius.max(dist + self.config.spawn_radius);
        }
        
        // Try multiple times to find a valid spawn position
        for _ in 0..30 {  // Increased attempts since we have more constraints now
            // Generate random angle and distance
            let angle = rng.next_f32() * std::f32::consts::TAU;
            let distance = self.config.min_player_distance + 
                rng.next_f32() * (enclosing_radius - self.config.min_player_distance);
            
            // Calculate potential spawn position
            let offset = Vec2::new(angle.cos(), angle.sin()) * distance;
            let spawn_pos = center + offset;
            
            // Check if position is valid (not too close to any player)
            let mut valid_position = true;
            for player_pos in player_positions {
                if spawn_pos.distance(*player_pos) < self.config.min_player_distance {
                    valid_position = false;
                    break;
                }
            }
            
            if !valid_position {
                continue;
            }
            
            // Create a test collider for the potential spawn
            /*
            let spawn_collider = Collider {
                shape: ColliderShape::Circle { radius: 30.0 },
                offset: Vec2::ZERO,
            };
            */
            
            let spawn_transform = Transform::from_translation(
                round_vec3(Vec3::new(spawn_pos.x, spawn_pos.y, 0.0))
            );
            
            // Check collision with existing enemies
            for (_, zombie_transform) in enemy_query.iter() {
                /*
                if is_colliding(
                    &spawn_transform,
                    &spawn_collider,
                    zombie_transform,
                    zombie_collider
                ) {
                    valid_position = false;
                    break;
                }
                */
            }
            
            // Check collision with walls
            if valid_position {
                for (_, wall_transform) in wall_query.iter() {
                    /*
                    if is_colliding(
                        &spawn_transform,
                        &spawn_collider,
                        wall_transform,
                        wall_collider
                    ) {
                        valid_position = false;
                        break;
                    }
                    */
                }
            }
            
            if valid_position {
                return Some(spawn_pos);
            }
        }
        
        None  // Couldn't find a valid position
    }
        
    
}

// Main spawn system that delegates to the configured spawn system
