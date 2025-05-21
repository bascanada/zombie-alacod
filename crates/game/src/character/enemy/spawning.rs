use animation::SpriteSheetConfig;
use bevy::{prelude::*};
use map::game::entity::map::enemy_spawn::EnemySpawnerComponent;
use utils::rng::RollbackRng;

use crate::{character::{config::CharacterConfig, player::Player}, collider::{Collider, CollisionSettings, Wall}, frame::FrameCount, global_asset::GlobalAsset, weapons::WeaponsConfig};

use super::{create::spawn_enemy, Enemy};

#[derive(Component, Debug, Reflect, Clone)]
#[reflect]
pub struct EnemySpawnerState {
    pub cooldown_remaining: u32,
    pub last_spawn_frame: u32,
    pub active: bool,
    pub current_enemies: u32,
}


impl Default for EnemySpawnerState {
    fn default() -> Self {
        Self {
            cooldown_remaining: 0,
            last_spawn_frame: 0,
            current_enemies: 0,
            active: true,
        }
    } 
}

pub fn enemy_spawn_from_spawners_system(
    mut commands: Commands,
    frame: Res<FrameCount>,
    mut rng: ResMut<RollbackRng>,
    mut spawner_query: Query<(Entity, &EnemySpawnerComponent, &mut EnemySpawnerState, &Transform)>,
    enemy_query: Query<&Transform, With<Enemy>>,
    player_query: Query<&Transform, With<Player>>,

    global_assets: Res<GlobalAsset>,
    collision_settings: Res<CollisionSettings>,
    weapons_asset: Res<Assets<WeaponsConfig>>,
    characters_asset: Res<Assets<CharacterConfig>>,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    sprint_sheet_assets: Res<Assets<SpriteSheetConfig>>,
) {
    // Get player positions for checking distance
    let player_positions: Vec<Vec2> = player_query
        .iter()
        .map(|transform| transform.translation.truncate())
        .collect();
    
    if player_positions.is_empty() {
        return; // No players, don't spawn
    }
    
    // Count current enemies (global count)
    let current_enemies = enemy_query.iter().count();
    let global_max_enemies = 20; // This could be a global config
    
    if current_enemies >= global_max_enemies {
        return; // Already at global max enemies
    }
    
    // Process each spawner
    for (_, config, mut state, transform) in spawner_query.iter_mut() {
        // Skip inactive spawners or those on cooldown
        if !state.active || state.cooldown_remaining > 0 {
            // Decrease cooldown
            if state.cooldown_remaining > 0 {
                state.cooldown_remaining -= 1;
            }
            continue;
        }
        
        // Skip if this spawner has reached its individual max
        if state.current_enemies >= config.max_enemies {
            continue;
        }
        
        let spawner_pos = transform.translation.truncate();
        
        // Check minimum distance to players
        let min_distance_to_player = player_positions.iter()
            .map(|pos| spawner_pos.distance(*pos))
            .min_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .unwrap_or(f32::MAX);
        
        // Don't spawn if too close to a player
        if min_distance_to_player < config.min_spawn_distance {
            continue;
        }
        
        // Calculate final spawn position (with optional small random offset)
        let spawn_pos = if config.spawn_radius > 0.0 {
            // Create deterministic offset using RNG
            let angle = rng.next_f32() * std::f32::consts::TAU;
            let distance = rng.next_f32() * config.spawn_radius;
            let offset = Vec2::new(angle.cos(), angle.sin()) * distance;
            
            // Apply the offset
            Vec3::new(
                spawner_pos.x + offset.x,
                spawner_pos.y + offset.y,
                0.0
            )
        } else {
            // Use exact spawner position
            transform.translation
        };
        
        // Select enemy type deterministically
        let type_index = (rng.next_u32() as usize) % config.enemy_types.len();
        let enemy_type_name = config.enemy_types[type_index].clone();
        
        // Spawn the enemy
        spawn_enemy(
            enemy_type_name,
            spawn_pos,
            &mut commands,
            &weapons_asset,
            &characters_asset,
            &asset_server,
            &mut texture_atlas_layouts,
            &sprint_sheet_assets,
            &global_assets,
            &collision_settings,
        );
        
        // Update state
        state.cooldown_remaining = config.max_cooldown;
        state.last_spawn_frame = frame.frame;
        state.current_enemies += 1;
        
        // We only spawn one enemy per system call
        break;
    }
}