pub mod ui;

use bevy::{prelude::*, scene::ron::de};
use bevy_ggrs::Rollback;
use ggrs::PlayerHandle;
use pathfinding::matrix::directions::N;
use serde::{Deserialize, Serialize};


#[derive(Component, Reflect, Debug, Clone, Serialize, Deserialize)]
pub enum HitBy {
    Entity(Entity),
    Player(PlayerHandle),
}


#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HealthConfig {
    pub max: f32,
}


#[derive(Component, Reflect, Clone, Debug, Serialize, Default, Deserialize)]
pub struct Health {
    pub current: f32,
    pub max: f32,
    pub invulnerable_until_frame: Option<u32>,  // Optional invulnerability window
}


#[derive(Component, Clone, Debug, Serialize, Deserialize, Default)]
pub struct Death {
    pub last_hit_by: Option<HitBy>,
}

#[derive(Component, Reflect, Clone, Serialize, Deserialize, Default)]
pub struct DamageAccumulator {
    pub total_damage: f32,
    pub hit_count: u32,
    pub last_hit_by: Option<HitBy>,
}

impl From<HealthConfig> for Health {
    fn from(value: HealthConfig) -> Self {
       Self { current: value.max, max: value.max, invulnerable_until_frame: None } 
    } 
}


pub fn rollback_apply_accumulated_damage(
    mut commands: Commands,
    mut query: Query<(Entity, &mut DamageAccumulator, &mut Health), With<Rollback>>,
) {
    for (entity, mut accumulator, mut health) in query.iter_mut() {

        if accumulator.total_damage > 0. {

            health.current -= accumulator.total_damage;
            
            accumulator.total_damage = 0.;
            accumulator.hit_count = 0;
            accumulator.last_hit_by = None;

            if health.current <= 0. {
                commands.entity(entity).insert(Death{ last_hit_by: accumulator.last_hit_by.clone( )});
                return;
            }
        }
    }
}


pub fn rollback_apply_death(
    mut commands: Commands,
    mut query: Query<(Entity, &Death), With<Rollback>>,
) {
    for (entity, death) in query.iter_mut() {
        info!("Entity {} killed by {:?}", entity, death.last_hit_by);
        commands.entity(entity).despawn_recursive();
    }
}

