use bevy::prelude::*;

use crate::weapons::Bullet;

use super::{Collider, CollisionLayer};

pub fn debug_draw_colliders_system(
    mut gizmos: Gizmos,
    collider_query: Query<(&Transform, &Collider, &CollisionLayer)>,
    bullet_query: Query<(&Transform, &Collider), With<Bullet>>,
) {
    // Draw regular colliders
    for (transform, collider, layer) in collider_query.iter() {
        let color = match layer.0 {
            0 => LinearRgba::RED,        // Bullets
            1 => LinearRgba::GREEN,      // Enemies
            2 => LinearRgba::BLUE,       // Environment
            3 => LinearRgba::WHITE,     // Players
            _ => LinearRgba::BLACK,       // Unknown
        };
        
        gizmos.circle_2d(
            transform.translation.truncate(),
            collider.radius,
            color,
        );
    }
    
    // Draw bullet colliders in a different style
    for (transform, collider) in bullet_query.iter() {
        gizmos.circle_2d(
            transform.translation.truncate(),
            collider.radius,
            LinearRgba::RED,
        );
    }
}
