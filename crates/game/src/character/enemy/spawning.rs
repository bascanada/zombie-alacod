// crates/game/src/enemy/config.rs
use bevy::prelude::*;
use serde::Deserialize;
use std::ops::Range;

#[derive(Asset, TypePath, Deserialize, Debug, Clone)]
pub struct BasicEnemySpawnConfig {
    pub timeout_range: Range<f32>,  // Range of seconds between spawns
    pub max_enemies: u32,           // Maximum number of enemies alive at once
    pub spawn_radius: f32,          // Radius around players where zombies can spawn
    pub min_player_distance: f32,   // Minimum distance from any player
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
pub enum EnemySpawnSystemType {
    BasicSpawnSystem,  // Spawn in circle encompassing all players
}

impl Default for BasicEnemySpawnConfig {
    fn default() -> Self {
        Self {
            timeout_range: 1.0..3.0,
            max_enemies: 10,
            spawn_radius: 500.0,
            min_player_distance: 100.0,
        }
    }
}