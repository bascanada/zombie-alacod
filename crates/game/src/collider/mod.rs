use bevy::prelude::*;
use avian2d::prelude::*;
use bevy_ggrs::AddRollbackCommandExtension;
use map::game::entity::map::wall::Wall;
use serde::{Deserialize, Serialize};


pub const PLAYER_LAYER: u32 = 1 << 0; // Layer 0
pub const WALL_LAYER: u32 = 1 << 1;   // Layer 1
pub const ENEMY_LAYER: u32 = 1 << 2;  // Layer 2

#[derive(Resource, Debug, Clone)]
pub struct CollisionLayerSettings {
    pub player: CollisionLayers,
    pub wall: CollisionLayers,
    pub enemy: CollisionLayers,
}

impl Default for CollisionLayerSettings {
    fn default() -> Self {
        Self {
            // Player belongs to PLAYER_LAYER and collides with WALL_LAYER and ENEMY_LAYER
            player: CollisionLayers::new(PLAYER_LAYER, WALL_LAYER | ENEMY_LAYER),
            // Wall belongs to WALL_LAYER and collides with PLAYER_LAYER and ENEMY_LAYER
            wall: CollisionLayers::new(WALL_LAYER, PLAYER_LAYER | ENEMY_LAYER),
            // Enemy belongs to ENEMY_LAYER and collides with PLAYER_LAYER and WALL_LAYER
            enemy: CollisionLayers::new(ENEMY_LAYER, PLAYER_LAYER | WALL_LAYER),
        }
    }
}


fn detect_collisions_system(
    mut collision_events: EventReader<CollisionStarted>,
    // Query for your entity types or other components to react to collisions
) {
    for CollisionStarted(entity_a, entity_b) in collision_events.read() {
        println!("collission between {} and {}", entity_a, entity_b);
    }
}


// test function for wall

pub fn spawn_test_wall(
    commands: &mut Commands,
    position: Vec3,
    size: Vec2,
    layer_settings: &Res<CollisionLayerSettings>, // Use the Avian-style layer settings
    color: Color,
) -> Entity {
    let wall_entity = commands.spawn((
        Wall::default(),
        Transform::from_translation(position),
        Sprite {
            color,
            custom_size: Some(size),
            ..Default::default()
        },
        RigidBody::Static, // Makes it a static physics body
        avian2d::prelude::Collider::rectangle(size.x, size.y), // Avian's Collider shape
        layer_settings.wall.clone(), // Assign collision layers for the wall
    ))
    .add_rollback()
    .id();


    wall_entity
}
