use animation::SpriteSheetConfig;
// crates/game/src/enemy/config.rs
use bevy::{prelude::*, utils::HashSet};
use utils::rng::RollbackRng;

use crate::{character::{config::CharacterConfig, player::Player}, collider::{Collider, CollisionSettings, Wall}, frame::FrameCount, global_asset::GlobalAsset, weapons::WeaponsConfig};

use super::{create::spawn_enemy, spawning_basic::{BasicEnemySpawnConfig, BasicEnemySpawnSystem}, Enemy};

#[derive(Resource, Default, Debug, Reflect, Clone)]
#[reflect]
pub struct EnemySpawnState {
    pub next_spawn_frame: u32,
}

impl std::hash::Hash for EnemySpawnState {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.next_spawn_frame.hash(state);
    }
}





// Trait for different spawn systems
pub trait EnemySpawnSystem: Send + Sync + 'static {
    fn should_spawn(&self, frame: &FrameCount, state: &EnemySpawnState, current_enemies: u32) -> bool;
    fn calculate_next_spawn_frame(&self, frame: &FrameCount, rng: &mut RollbackRng) -> u32;
    fn calculate_spawn_position(
        &self,
        rng: &mut RollbackRng,
        player_positions: &[Vec2],
        enemy_query: &Query<(Entity, &Transform, &Collider), With<Enemy>>,
        wall_query: &Query<(Entity, &Transform, &Collider), With<Wall>>,
    ) -> Option<Vec2>;
}


#[derive(Resource)]
pub struct EnemySpawnSystemHolder {
    pub system: Box<dyn EnemySpawnSystem>,
}

impl Default for EnemySpawnSystemHolder {
    fn default() -> Self {
        Self { system: Box::new(BasicEnemySpawnSystem::new(BasicEnemySpawnConfig::default())) }
    } 
}



pub fn enemy_spawn_system(
    mut commands: Commands,
    frame: Res<FrameCount>,
    mut spawn_state: ResMut<EnemySpawnState>,
    mut rng: ResMut<RollbackRng>,
    spawn_system_holder: Res<EnemySpawnSystemHolder>,
    player_query: Query<(&Transform, &Player)>,
    zombie_query: Query<(Entity, &Transform, &Collider), With<Enemy>>,
    wall_query: Query<(Entity, &Transform, &Collider), With<Wall>>,
    collision_settings: Res<CollisionSettings>,

    weapons_asset: Res<Assets<WeaponsConfig>>,
    characters_asset: Res<Assets<CharacterConfig>>,

    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    sprint_sheet_assets: Res<Assets<SpriteSheetConfig>>,

    global_assets: Res<GlobalAsset>,
) {

    let spawn_system = &spawn_system_holder.system;
    
    // Count current zombies
    let current_zombies = zombie_query.iter().count() as u32;
    
    // Check if we should spawn more zombies
    if !spawn_system.should_spawn(&frame, &spawn_state, current_zombies) {
        return;
    }
    
    // Calculate next spawn frame
    spawn_state.next_spawn_frame = spawn_system.calculate_next_spawn_frame(&frame, &mut rng);

    
    // Collect all player positions
    let player_positions: Vec<Vec2> = player_query
        .iter()
        .map(|(transform, _)| transform.translation.truncate())
        .collect();
    
    // Calculate spawn position
    if let Some(spawn_pos) = spawn_system.calculate_spawn_position(&mut rng, &player_positions, &zombie_query, &wall_query) {
        spawn_enemy("zombie_1".into(), spawn_pos.extend(0.0), &mut commands, &weapons_asset, &characters_asset, &asset_server, &mut texture_atlas_layouts, &sprint_sheet_assets, &global_assets, &collision_settings);
    }
}