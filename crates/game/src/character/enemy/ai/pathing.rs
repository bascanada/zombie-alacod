use animation::FacingDirection;
// crates/game/src/enemy/path.rs
use bevy::prelude::*;
use utils::rng::RollbackRng;
use std::collections::VecDeque;
use crate::character::config::{CharacterConfig, CharacterConfigHandles};
use crate::character::enemy::Enemy;
use crate::character::movement::Velocity;
use crate::character::player::Player;
use crate::collider::{Collider, is_colliding, Wall};
use crate::frame::FrameCount;


#[derive(Component, Debug, Clone, Reflect, Default)]
#[reflect(Component)]
pub struct EnemyPath {
    // Target to move toward
    pub target_position: Vec2,
    // Queue of waypoints (if using pathfinding)
    pub waypoints: VecDeque<Vec2>,
    // Path recalculation timer 
    pub recalculate_ticks: u32,
    // Path status
    pub path_status: PathStatus,
}

#[derive(Debug, Clone, Reflect, PartialEq, Eq, Default)]
pub enum PathStatus {
    #[default]
    Idle,
    DirectPath,
    CalculatingPath,
    FollowingPath,
    Blocked,
}

#[derive(Resource, Reflect, Clone)]
#[reflect(Resource)]
pub struct PathfindingConfig {
    // How often to recalculate paths (in frames)
    pub recalculation_interval: u32,
    // Maximum pathfinding iterations
    pub max_iterations: u32,
    // Maximum path length
    pub max_path_length: usize,
    // Direct path threshold (distance at which to use direct path)
    pub direct_path_threshold: f32,
    // Node size for discretization (if using grid-based approach)
    pub node_size: f32,
    // Movement speed fallback
    pub movement_speed: f32,
    // Waypoint reach distance
    pub waypoint_reach_distance: f32,
    // Minimum distance to maintain from player (attack range)
    pub optimal_attack_distance: f32,
    // Distance at which to start slowing down
    pub slow_down_distance: f32,
    // Separation force between enemies
    pub enemy_separation_force: f32,
    // Separation distance between enemies
    pub enemy_separation_distance: f32,
}

impl Default for PathfindingConfig {
    fn default() -> Self {
        Self {
            recalculation_interval: 30, // Recalculate every half second (at 60 FPS)
            max_iterations: 1000,
            max_path_length: 50,
            direct_path_threshold: 200.0,
            node_size: 20.0,
            movement_speed: 20.0,
            waypoint_reach_distance: 10.0,
            optimal_attack_distance: 100.0,     // Keep this distance from players
            slow_down_distance: 150.0,          // Start slowing down at this distance
            enemy_separation_force: 2.0,        // Much stronger separation force
            enemy_separation_distance: 80.0,    // Larger separation distance
        }
    }
}

// System to find closest player and set as target
pub fn update_enemy_targets(
    player_query: Query<(&Transform, &Player)>,
    mut enemy_query: Query<(&Transform, &mut EnemyPath), With<Enemy>>,
    frame: Res<FrameCount>,
    config: Res<PathfindingConfig>,
) {
    // Get all player positions
    let player_positions: Vec<Vec2> = player_query
        .iter()
        .map(|(transform, _)| transform.translation.truncate())
        .collect();
    
    if player_positions.is_empty() {
        return;
    }
    
    // Update each enemy's target
    for (transform, mut path) in enemy_query.iter_mut() {
        // Only update periodically to save performance
        if frame.frame % config.recalculation_interval != 0 {
            continue;
        }
        
        // Find the closest player
        let enemy_pos = transform.translation.truncate();
        let mut closest_player = player_positions[0];
        let mut closest_distance = enemy_pos.distance(closest_player);
        
        for pos in &player_positions[1..] {
            let distance = enemy_pos.distance(*pos);
            if distance < closest_distance {
                closest_distance = distance;
                closest_player = *pos;
            }
        }
        
        // Set the target
        path.target_position = closest_player;
        
        // Mark for path recalculation
        path.recalculate_ticks = frame.frame;
    }
}

// System to check if direct path is clear
pub fn check_direct_paths(
    wall_query: Query<(&Transform, &Collider), With<Wall>>,
    mut enemy_query: Query<(&Transform, &mut EnemyPath), With<Enemy>>,
    config: Res<PathfindingConfig>,
) {
    for (transform, mut path) in enemy_query.iter_mut() {
        let enemy_pos = transform.translation.truncate();
        let target = path.target_position;
        
        // If target is close enough, use direct path
        let distance = enemy_pos.distance(target);
        if distance < config.direct_path_threshold {
            // Check for wall collisions along direct path
            let direction = (target - enemy_pos).normalize();
            let steps = (distance / 20.0).ceil() as usize;
            let step_size = distance / steps as f32;
            
            let mut path_blocked = false;
            
            // Virtual collider for checking along the path
            let test_collider = Collider {
                shape: crate::collider::ColliderShape::Circle { radius: 15.0 },
                offset: Vec2::ZERO,
            };
            
            // Check several points along the path
            for i in 1..steps {
                let test_pos = enemy_pos + direction * (step_size * i as f32);
                let test_transform = Transform::from_translation(test_pos.extend(0.0));
                
                for (wall_transform, wall_collider) in wall_query.iter() {
                    if is_colliding(&test_transform, &test_collider, wall_transform, wall_collider) {
                        path_blocked = true;
                        break;
                    }
                }
                
                if path_blocked {
                    break;
                }
            }
            
            if !path_blocked {
                // Clear any existing waypoints and use direct path
                path.waypoints.clear();
                path.path_status = PathStatus::DirectPath;
            } else if path.path_status != PathStatus::FollowingPath {
                // Need to calculate a path around obstacles
                path.path_status = PathStatus::CalculatingPath;
            }
        } else {
            // Target is far, calculate full path
            path.path_status = PathStatus::CalculatingPath;
        }
    }
}

// System to calculate paths around obstacles when needed
pub fn calculate_paths(
    mut enemy_query: Query<(&Transform, &mut EnemyPath), With<Enemy>>,
    wall_query: Query<(&Transform, &Collider), With<Wall>>,
    mut rng: ResMut<RollbackRng>,
    config: Res<PathfindingConfig>, 
) {
    for (transform, mut path) in enemy_query.iter_mut() {
        if path.path_status != PathStatus::CalculatingPath {
            continue;
        }
        
        let enemy_pos = transform.translation.truncate();
        let target = path.target_position;
        
        // Simplified A* pathfinding using vector-based movement:
        // This is a simplified version that works well for games with fewer obstacles
        let mut waypoints = VecDeque::new();
        let direct_dir = (target - enemy_pos).normalize();
        
        // Try several angles to find a clear path
        let base_angles = [0.0, 0.5, -0.5, 1.0, -1.0, 1.5, -1.5];
        let mut best_angle = 0.0;
        let mut best_clearance = 0.0;
        
        for angle_offset in base_angles {
            let angle = direct_dir.y.atan2(direct_dir.x) + angle_offset;
            let test_dir = Vec2::new(angle.cos(), angle.sin());
            
            // Check how far we can go in this direction
            let mut max_distance = 0.0;
            let step_size = 20.0;
            let test_collider = Collider {
                shape: crate::collider::ColliderShape::Circle { radius: 15.0 },
                offset: Vec2::ZERO,
            };
            
            for i in 1..20 {
                let test_dist = i as f32 * step_size;
                let test_pos = enemy_pos + test_dir * test_dist;
                let test_transform = Transform::from_translation(test_pos.extend(0.0));
                
                let mut collides = false;
                for (wall_transform, wall_collider) in wall_query.iter() {
                    if is_colliding(&test_transform, &test_collider, wall_transform, wall_collider) {
                        collides = true;
                        break;
                    }
                }
                
                if collides {
                    max_distance = test_dist - step_size;
                    break;
                } else {
                    max_distance = test_dist;
                }
            }
            
            // Consider distance to target along this path
            let waypoint = enemy_pos + test_dir * max_distance;
            let remaining = target.distance(waypoint);
            let clearance = max_distance / (1.0 + remaining * 0.01);
            
            if clearance > best_clearance {
                best_clearance = clearance;
                best_angle = angle;
            }
        }
        
        // Found a good direction - create waypoint
        let best_dir = Vec2::new(best_angle.cos(), best_angle.sin());
        let distance = (best_clearance * 0.8).min(100.0); // Don't go too far at once
        
        // Add a slight random variation to prevent getting stuck in patterns
        let jitter_angle = (rng.next_f32() - 0.5) * 0.2;
        let jitter_dir = Vec2::new((best_angle + jitter_angle).cos(), (best_angle + jitter_angle).sin());
        let waypoint = enemy_pos + jitter_dir * distance;
        
        waypoints.push_back(waypoint);
        
        // Optionally add the final target as the last waypoint
        if waypoints.len() < config.max_path_length {
            waypoints.push_back(target);
        }
        
        path.waypoints = waypoints;
        path.path_status = PathStatus::FollowingPath;
    }
}

pub fn move_enemies(
    mut enemy_query: Query<(
        Entity,
        &mut Transform,
        &mut Velocity, 
        &mut EnemyPath, 
        &mut FacingDirection,
        &CharacterConfigHandles
    ), With<Enemy>>,
    player_query: Query<&Transform, (With<Player>, Without<Enemy>)>,
    character_configs: Res<Assets<CharacterConfig>>,
    config: Res<PathfindingConfig>,
    time: Res<Time>,
) {
    // First pass - collect all enemy positions for separation calculation
    let enemy_positions: Vec<(Entity, Vec2)> = enemy_query
        .iter()
        .map(|(entity, transform, ..)| (entity, transform.translation.truncate()))
        .collect();
    
    // Second pass - calculate and apply movement
    for (entity, mut transform, mut velocity, mut path, mut facing_direction, config_handles) in enemy_query.iter_mut() {
        let enemy_pos = transform.translation.truncate();
        
        // Get character movement config
        let movement_speed = if let Some(char_config) = character_configs.get(&config_handles.config) {
            char_config.movement.max_speed
        } else {
            // Fallback speed if config not found
            config.movement_speed
        };
        
        // Check distance to target (either waypoint or final target)
        let target_pos = if let Some(waypoint) = path.waypoints.front() {
            *waypoint
        } else {
            path.target_position
        };
        
        // Direction to target
        let direction_to_target = (target_pos - enemy_pos).normalize_or_zero();
        
        // Calculate distance to nearest player (for attack range check)
        let mut distance_to_nearest_player = f32::MAX;
        for player_transform in player_query.iter() {
            let player_pos = player_transform.translation.truncate();
            let distance = enemy_pos.distance(player_pos);
            distance_to_nearest_player = distance_to_nearest_player.min(distance);
        }
        
        // Calculate separation force (avoid other enemies)
        let mut separation = Vec2::ZERO;
        let mut separation_count = 0;
        
        for (other_entity, other_pos) in &enemy_positions {
            // Skip self
            if *other_entity == entity {
                continue;
            }
            
            let distance = enemy_pos.distance(*other_pos);
            if distance < config.enemy_separation_distance && distance > 0.1 {
                // Calculate repulsion vector (away from other enemy)
                let repulsion = (enemy_pos - *other_pos).normalize() / distance.max(1.0);
                separation += repulsion;
                separation_count += 1;
            }
        }
        
        // Normalize and scale separation force
        if separation_count > 0 {
            separation = (separation / separation_count as f32) * config.enemy_separation_force;
        }
        
        // Base movement velocity
        let mut move_velocity = Vec2::ZERO;
        
        match path.path_status {
            PathStatus::DirectPath | PathStatus::FollowingPath => {
                // Calculate base velocity toward target (or waypoint)
                let base_velocity = direction_to_target * movement_speed;
                
                // Apply speed reduction based on proximity to target player
                let speed_factor = if distance_to_nearest_player < config.optimal_attack_distance {
                    // Too close - back up slightly
                    -0.3 
                } else if distance_to_nearest_player < config.slow_down_distance {
                    // Within slowing range - scale speed based on distance
                    let t = (distance_to_nearest_player - config.optimal_attack_distance) / 
                           (config.slow_down_distance - config.optimal_attack_distance);
                    t.clamp(0.0, 1.0)
                } else {
                    // Far away - full speed
                    1.0
                };
                
                move_velocity = base_velocity * speed_factor;
                
                // For FollowingPath, check if we've reached waypoint
                if let PathStatus::FollowingPath = path.path_status {
                    if let Some(waypoint) = path.waypoints.front() {
                        if enemy_pos.distance(*waypoint) < config.waypoint_reach_distance {
                            path.waypoints.pop_front();
                            
                            // If no more waypoints, go back to direct path
                            if path.waypoints.is_empty() {
                                path.path_status = PathStatus::DirectPath;
                            }
                        }
                    } else {
                        path.path_status = PathStatus::DirectPath;
                    }
                }
            },
            _ => {
                // Idle or calculating - don't move toward target
            }
        }
        
        // Combine movement and separation
        let final_velocity = move_velocity + separation;
        velocity.0 = final_velocity;
        
        // Apply movement
        if velocity.length_squared() > 0.01 {
            transform.translation.x += velocity.x * time.delta().as_secs_f32();
            transform.translation.y += velocity.y * time.delta().as_secs_f32();
            
            // Update facing direction based on movement
            if velocity.x > 0.1 {
                *facing_direction = FacingDirection::Right;
            } else if velocity.x < -0.1 {
                *facing_direction = FacingDirection::Left;
            }
        }
    }
}

fn update_facing_direction(facing_direction: &mut FacingDirection, velocity: &Velocity) {
    if velocity.x > 0.1 {
        *facing_direction = FacingDirection::Right;
    } else if velocity.x < -0.1 {
        *facing_direction = FacingDirection::Left;
    }
}