use bevy::prelude::*;

use super::Health;


#[derive(Component)]
pub struct HealthBar;

pub fn setup_health_bars(
    mut commands: Commands,
    query: Query<Entity, Added<Health>>,
) {
    for entity in query.iter() {
        // Create a health bar entity as a child of the entity
        commands.entity(entity).with_children(|parent| {
            parent.spawn((
                HealthBar,
                SpriteBundle {
                    sprite: Sprite {
                        color: LinearRgba::GREEN.into(),
                        custom_size: Some(Vec2::new(30.0, 3.0)),
                        ..default()
                    },
                    transform: Transform::from_translation(Vec3::new(0.0, 10.0, 0.1)),
                    ..default()
                },
            ));
        });
    }
}

pub fn update_health_bars(
    health_query: Query<(&Health, &Children)>,
    mut health_bar_query: Query<(&mut Transform, &mut Sprite), With<HealthBar>>,
) {
    for (health, children) in health_query.iter() {
        for child in children.iter() {
            if let Ok((mut transform, mut sprite)) = health_bar_query.get_mut(*child) {
                // Update the health bar size based on current/max health
                let health_ratio = health.current / health.max;
                sprite.custom_size = Some(Vec2::new(30.0 * health_ratio, 3.0));
                
                // Optional: Change color based on health
                /*
                sprite.color = if health_ratio > 0.6 {
                    LinearRgba::GREEN
                } else if health_ratio > 0.3 {
                    LinearRgba::GREEN
                } else {
                    LinearRgba::RED
                };*/
            }
        }
    }
}