pub mod ui;

use bevy::prelude::*;


#[derive(Component)]
pub struct Collider {
    pub radius: f32,  // Using circle colliders for simplicity
}

#[derive(Component)]
pub struct CollisionLayer(pub usize);




#[derive(Resource)]
pub struct CollisionSettings {
    pub bullet_layer: usize,
    pub enemy_layer: usize,
    pub environment_layer: usize,
    pub player_layer: usize,
    pub layer_matrix: [[bool; 8]; 8], // Collision matrix for which layers collide
}

impl Default for CollisionSettings {
    fn default() -> Self {
        // Initialize empty collision matrix
        let mut layer_matrix = [[false; 8]; 8];
        
        // Define collision layers
        let bullet_layer = 0;
        let enemy_layer = 1;
        let environment_layer = 2;
        let player_layer = 3;
        
        // Set up collision relationships
        layer_matrix[bullet_layer][enemy_layer] = true;
        layer_matrix[bullet_layer][environment_layer] = true;
        layer_matrix[enemy_layer][bullet_layer] = true; // Symmetric for simplicity
        layer_matrix[environment_layer][bullet_layer] = true;
        
        // Player bullets shouldn't hit players
        layer_matrix[bullet_layer][player_layer] = false;
        layer_matrix[player_layer][bullet_layer] = false;
        
        Self {
            bullet_layer,
            enemy_layer,
            environment_layer,
            player_layer,
            layer_matrix,
        }
    }
}
