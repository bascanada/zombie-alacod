// crates/game/src/enemy/spawn.rs
use bevy::prelude::*;
use crate::frame::FrameCount;
use utils::rng::RollbackRng;
use crate::character::{movement::Velocity, player::Player};
use crate::collider::{Collider, ColliderShape, CollisionLayer, CollisionSettings, is_colliding};
use bevy_ggrs::AddRollbackCommandExtension;

use super::spawning::BasicEnemySpawnConfig;
use super::Enemy;

#[derive(Resource, Default, Reflect, Hash, Clone, Copy)]
#[reflect(Hash)]
pub struct EnemySpawnState {
    pub next_spawn_frame: u32,
}

// Trait for different spawn systems
pub trait EnemySpawnSystem {
    fn should_spawn(&self, frame: &FrameCount, state: &EnemySpawnState, config: &BasicEnemySpawnConfig, current_enemies: u32) -> bool;
    fn calculate_next_spawn_frame(&self, frame: &FrameCount, rng: &mut RollbackRng, config: &BasicEnemySpawnConfig) -> u32;
    fn calculate_spawn_position(
        &self,
        rng: &mut RollbackRng,
        config: &BasicEnemySpawnConfig,
        player_positions: &[Vec2],
        zombie_query: &Query<(Entity, &Transform, &Collider), With<Enemy>>,
    ) -> Option<Vec2>;
}

// Implementation for spawning in a circle around players
pub struct CircleAroundPlayersSpawnSystem;
impl EnemySpawnSystem for CircleAroundPlayersSpawnSystem {
    fn should_spawn(&self, frame: &FrameCount, state: &EnemySpawnState, config: &BasicEnemySpawnConfig, current_enemies: u32) -> bool {
        current_enemies < config.max_enemies && frame.frame >= state.next_spawn_frame
    }
    
    fn calculate_next_spawn_frame(&self, frame: &FrameCount, rng: &mut RollbackRng, config: &BasicEnemySpawnConfig) -> u32 {
        let min_frames = (config.timeout_range.start * 60.0) as u32;
        let max_frames = (config.timeout_range.end * 60.0) as u32;
        let frame_delay = min_frames + (rng.next_f32() * (max_frames - min_frames) as f32) as u32;
        frame.frame + frame_delay
    }
    
    fn calculate_spawn_position(
        &self,
        rng: &mut RollbackRng,
        config: &BasicEnemySpawnConfig,
        player_positions: &[Vec2],
        zombie_query: &Query<(Entity, &Transform, &Collider), With<Enemy>>,
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
        let mut enclosing_radius = config.spawn_radius;
        for pos in player_positions {
            let dist = center.distance(*pos);
            enclosing_radius = enclosing_radius.max(dist + config.spawn_radius);
        }
        
        // Try multiple times to find a valid spawn position
        for _ in 0..20 {
            // Generate random angle and distance
            let angle = rng.next_f32() * std::f32::consts::TAU;
            let distance = config.min_player_distance + 
                rng.next_f32() * (enclosing_radius - config.min_player_distance);
            
            // Calculate potential spawn position
            let offset = Vec2::new(angle.cos(), angle.sin()) * distance;
            let spawn_pos = center + offset;
            
            // Check if position is valid (not too close to any player)
            let mut valid_position = true;
            for player_pos in player_positions {
                if spawn_pos.distance(*player_pos) < config.min_player_distance {
                    valid_position = false;
                    break;
                }
            }
            
            // Check if not overlapping with other zombies
            if valid_position {
                let spawn_collider = Collider {
                    shape: ColliderShape::Circle { radius: 30.0 },
                    offset: Vec2::ZERO,
                };
                
                let spawn_transform = Transform::from_translation(Vec3::new(spawn_pos.x, spawn_pos.y, 0.0));
                
                for (_, zombie_transform, zombie_collider) in zombie_query.iter() {
                    if is_colliding(
                        &spawn_transform,
                        &spawn_collider,
                        zombie_transform,
                        zombie_collider
                    ) {
                        valid_position = false;
                        break;
                    }
                }
            }
            
            if valid_position {
                return Some(spawn_pos);
            }
        }
        
        None  // Couldn't find a valid position
    }
}

// Factory function to get the appropriate spawn system based on config
pub fn get_spawn_system(config: &EnemySpawnConfig) -> Box<dyn EnemySpawnSystem> {
    match config.spawn_system_type {
        EnemySpawnSystemType::CircleAroundPlayers => Box::new(CircleAroundPlayersSpawnSystem),
        EnemySpawnSystemType::RandomOnMap => {
            // Fallback until RandomOnMap is implemented
            Box::new(CircleAroundPlayersSpawnSystem)
        }
    }
}

// Main spawn system that delegates to the configured spawn system
pub fn enemy_spawn_system(
    mut commands: Commands,
    frame: Res<FrameCount>,
    mut spawn_state: ResMut<EnemySpawnState>,
    config: Res<EnemySpawnConfig>,
    mut rng: ResMut<RollbackRng>,
    player_query: Query<(&Transform, &Player)>,
    zombie_query: Query<(Entity, &Transform, &Collider), With<ZombieEnemy>>,
    collision_settings: Res<CollisionSettings>,
) {
    let spawn_system = get_spawn_system(&config);
    
    // Count current zombies
    let current_zombies = zombie_query.iter().count() as u32;
    
    // Check if we should spawn more zombies
    if !spawn_system.should_spawn(&frame, &spawn_state, &config, current_zombies) {
        return;
    }
    
    // Calculate next spawn frame
    spawn_state.next_spawn_frame = spawn_system.calculate_next_spawn_frame(&frame, &mut rng, &config);
    
    // Collect all player positions
    let player_positions: Vec<Vec2> = player_query
        .iter()
        .map(|(transform, _)| transform.translation.truncate())
        .collect();
    
    // Calculate spawn position
    if let Some(spawn_pos) = spawn_system.calculate_spawn_position(&mut rng, &config, &player_positions, &zombie_query) {
        spawn_zombie(&mut commands, spawn_pos, &collision_settings);
    }
}