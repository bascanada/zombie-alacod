use bevy::prelude::*;
use bevy_ggrs::AddRollbackCommandExtension;

use crate::{character::movement::Velocity, frame::FrameCount};

#[derive(Component, Reflect, Default, Debug, Copy, Clone)]
#[reflect(Component)]
pub struct Weight {
    pub mass: f32,

    pub factor: Option<f32>,
}



/// Apply a physics push to a target based on the impacting entity's properties
/// 
/// # Arguments
/// * `impact_mass` - Mass of the impacting entity
/// * `impact_velocity` - Velocity vector of the impacting entity
/// * `target_mass` - Mass of the target entity
/// * `push_factor` - Scaling factor to adjust game feel (0.0-1.0)
/// 
/// # Returns
/// The calculated push velocity to apply to the target
pub fn calculate_physics_push(
    impact_mass: f32,
    impact_velocity: Vec2,
    target_mass: f32,
    push_factor: f32,
) -> Vec2 {
    // Basic momentum transfer: p = mv
    let impact_momentum = impact_velocity * impact_mass;
    
    // Calculate the resulting velocity based on target mass
    // Scale by push_factor to tune the game feel
    let base_push = impact_momentum / target_mass.max(0.1) * push_factor;
    
    // Apply a reasonable maximum push limit to prevent extreme effects
    let max_push_magnitude = 15.0;
    if base_push.length_squared() > max_push_magnitude * max_push_magnitude {
        base_push.normalize() * max_push_magnitude
    } else {
        base_push
    }
}

/// Apply push force to an entity with velocity using frame-based duration
pub fn apply_push_to_entity(
    commands: &mut Commands,
    entity: Entity,
    push_velocity: Vec2,
    frame_count: &crate::frame::FrameCount,
) {
    // For a 60 FPS game, 6 frames is approximately 0.1 seconds
    const PUSH_DURATION_FRAMES: u32 = 6;
    
    commands.entity(entity).add_rollback().insert(PushImpact {
        velocity: push_velocity,
        duration_frames: PUSH_DURATION_FRAMES,
        start_frame: frame_count.frame,
    });
}

/// Component for temporary push impact effect using frames
#[derive(Component, Clone)]
pub struct PushImpact {
    pub velocity: Vec2,
    pub duration_frames: u32,
    pub start_frame: u32,
}

/// System to handle push impact effects using frame count
pub fn process_push_impacts(
    mut commands: Commands,
    frame_count: Res<crate::frame::FrameCount>,
    mut query: Query<(Entity, &PushImpact, &mut Velocity)>,
) {
    let current_frame = frame_count.frame;
    
    for (entity, push_impact, mut velocity) in query.iter_mut() {
        // Calculate how many frames have passed
        let frames_elapsed = current_frame.saturating_sub(push_impact.start_frame);
        
        if frames_elapsed < push_impact.duration_frames {
            // Apply push impact to velocity
            // We apply a decreasing factor as the effect approaches its end
            let remaining_factor = 1.0 - (frames_elapsed as f32 / push_impact.duration_frames as f32);
            velocity.0 += push_impact.velocity * remaining_factor;
        } else {
            // Remove component when duration is over
            commands.entity(entity).remove::<PushImpact>();
        }
    }
}



#[derive(Component, Clone)]
pub struct PushAccumulator {
    /// Total accumulated velocity from all impacts in this frame
    pub total_velocity: Vec2,
    /// Number of impacts accumulated
    pub count: u32,
    /// The frame this accumulator was created on
    pub frame: u32,
}


pub fn process_push_accumulators(
    mut commands: Commands,
    frame_count: Res<FrameCount>,
    query: Query<(Entity, &PushAccumulator)>,
) {
    for (entity, accumulator) in query.iter() {
        // Only process accumulators from the current frame
        if accumulator.frame == frame_count.frame {
            // If we've accumulated multiple impacts, average them and apply
            // This prevents excessive force when hit by multiple bullets
            let final_velocity = if accumulator.count > 1 {
                // Use a diminishing returns formula for multiple impacts
                // This ensures entities don't get pushed with unrealistic force
                let diminishing_factor = 1.0 / (0.5 * accumulator.count as f32).sqrt();
                accumulator.total_velocity * diminishing_factor
            } else {
                accumulator.total_velocity
            };

            // Apply the push impact
            apply_push_to_entity(
                &mut commands,
                entity,
                final_velocity,
                &frame_count
            );
        }

        // Always remove the accumulator after processing
        commands.entity(entity).remove::<PushAccumulator>();
    }
}