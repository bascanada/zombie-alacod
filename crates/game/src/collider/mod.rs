use bevy::prelude::*;
use bevy_ggrs::AddRollbackCommandExtension;
use serde::{Deserialize, Serialize};


#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ColliderShape {
    Circle { radius: f32 },
    Rectangle { width: f32, height: f32 },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ColliderConfig {
    pub shape: ColliderShape,
    pub offset: (f32, f32), // Offset from entity transform
}

impl Into<Collider> for &ColliderConfig {
    fn into(self) -> Collider {
        Collider { shape: self.shape.clone(), offset: Vec2::new(self.offset.0, self.offset.1)}
    } 
}



#[derive(Component, Clone, Debug, Serialize, Deserialize)]
pub struct Collider {
    pub shape: ColliderShape,
    pub offset: Vec2, // Offset from entity transform
}


#[derive(Component, Clone)]
pub struct Wall;


#[derive(Component, Clone, Serialize, Deserialize)]
pub struct CollisionLayer(pub usize);


#[derive(Resource)]
pub struct CollisionSettings {
    pub enemy_layer: usize,
    pub environment_layer: usize,
    pub player_layer: usize,
    pub wall_layer: usize,
    pub layer_matrix: [[bool; 8]; 8], // Collision matrix for which layers collide
}

impl Default for CollisionSettings {
    fn default() -> Self {
        // Initialize empty collision matrix
        let mut layer_matrix = [[false; 8]; 8];
        
        // Define collision layers
        let enemy_layer = 1;
        let environment_layer = 2;
        let player_layer = 3;
        let wall_layer = 4;
        
        // Set up collision relationships
        layer_matrix[enemy_layer][wall_layer] = true; // Symmetric for simplicity
        layer_matrix[enemy_layer][player_layer] = true; // Symmetric for simplicity
        layer_matrix[player_layer][enemy_layer] = true;

        layer_matrix[wall_layer][enemy_layer] = true;
        layer_matrix[wall_layer][player_layer] = true;
        layer_matrix[player_layer][wall_layer] = true;
        
        // Player bullets shouldn't hit players
        
        Self {
            enemy_layer,
            environment_layer,
            player_layer,
            wall_layer,
            layer_matrix,
        }
    }
}


pub fn is_colliding(
    transform_a: &Transform,
    collider_a: &Collider,
    transform_b: &Transform,
    collider_b: &Collider,
) -> bool {
    let pos_a = transform_a.translation.truncate() + collider_a.offset;
    let pos_b = transform_b.translation.truncate() + collider_b.offset;

    match (&collider_a.shape, &collider_b.shape) {
        // Circle to Circle
        (ColliderShape::Circle { radius: radius_a }, ColliderShape::Circle { radius: radius_b }) => {
            let distance = pos_a.distance(pos_b);
            distance < radius_a + radius_b
        },
        
        // Rectangle to Rectangle
        (ColliderShape::Rectangle { width: width_a, height: height_a }, 
         ColliderShape::Rectangle { width: width_b, height: height_b }) => {
            // AABB collision check
            let half_size_a = Vec2::new(width_a / 2.0, height_a / 2.0);
            let half_size_b = Vec2::new(width_b / 2.0, height_b / 2.0);
            
            let min_a = pos_a - half_size_a;
            let max_a = pos_a + half_size_a;
            let min_b = pos_b - half_size_b;
            let max_b = pos_b + half_size_b;
            
            // Check for overlap
            min_a.x <= max_b.x && 
            max_a.x >= min_b.x && 
            min_a.y <= max_b.y && 
            max_a.y >= min_b.y
        },
        
        // Circle to Rectangle
        (ColliderShape::Circle { radius }, 
         ColliderShape::Rectangle { width, height }) => {
            circle_rect_collision(pos_a, *radius, pos_b, *width, *height)
        },
        
        // Rectangle to Circle (swap arguments)
        (ColliderShape::Rectangle { width, height }, 
         ColliderShape::Circle { radius }) => {
            circle_rect_collision(pos_b, *radius, pos_a, *width, *height)
        },
    }
}

// Helper function for circle-to-rectangle collision
pub fn circle_rect_collision(
    circle_pos: Vec2,
    circle_radius: f32,
    rect_pos: Vec2,
    rect_width: f32,
    rect_height: f32,
) -> bool {
    // Calculate half-size of rectangle
    let half_width = rect_width / 2.0;
    let half_height = rect_height / 2.0;
    
    // Find the closest point on the rectangle to the circle center
    let closest_x = circle_pos.x.max(rect_pos.x - half_width).min(rect_pos.x + half_width);
    let closest_y = circle_pos.y.max(rect_pos.y - half_height).min(rect_pos.y + half_height);
    
    // Calculate distance from circle center to closest point
    let distance = Vec2::new(circle_pos.x - closest_x, circle_pos.y - closest_y).length();
    
    // Circle and rectangle collide if this distance is less than the circle radius
    distance < circle_radius
}



// test function for wall

pub fn spawn_test_wall(
    commands: &mut Commands,
    position: Vec3,
    size: Vec2,
    collision_settings: &Res<CollisionSettings>,
    color: Color,
) {
    commands.spawn((
        Wall,
        Transform::from_translation(position),
        Sprite {
            color,
            custom_size: Some(size),
            ..Default::default()
        },
        Collider {
            shape: ColliderShape::Rectangle { 
                width: size.x, 
                height: size.y 
            },
            offset: Vec2::ZERO,
        },
        CollisionLayer(collision_settings.wall_layer),
    )).add_rollback();
}
