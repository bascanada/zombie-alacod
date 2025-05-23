
use animation::{ActiveLayers, FacingDirection};
use animation::{AnimationState, CharacterAnimationHandles};
use avian2d::prelude::*;
use bevy::window::PrimaryWindow;
use bevy::{prelude::*, time::Time, utils::HashMap};
use leafwing_input_manager::prelude::*;
use bevy_ggrs::prelude::*;
use bevy_ggrs::LocalInputs;
use serde::{Serialize, Deserialize}; 

use crate::character::config::{CharacterConfig, CharacterConfigHandles};
use crate::character::dash::DashState;
use crate::character::movement::{FrameMovementIntent, SprintState};
use crate::character::player::{control::PlayerAction, Player};
use crate::weapons::WeaponInventory;

use super::jjrs::PeerConfig;
use super::LocalPlayer;

pub const FIXED_TIMESTEP: f32 = 1.0 / 60.0; // 60 FPS fixed timestep


const INPUT_UP: u16 = 1 << 0;
const INPUT_DOWN: u16 = 1 << 1;
const INPUT_LEFT: u16 = 1 << 2;
const INPUT_RIGHT: u16 = 1 << 3;
pub const INPUT_RELOAD: u16 = 1 << 4;
pub const INPUT_SWITCH_WEAPON_MODE: u16 = 1 << 5;
pub const INPUT_SPRINT: u16 = 1 << 6;
pub const INPUT_DASH: u16 = 1 << 7;
pub const INPUT_MODIFIER: u16 = 1 << 8;

const PAN_FACING_THRESHOLD: i16 = 5;

#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct BoxInput{
    pub buttons: u16,
    pub pan_x: i16,
    pub pan_y: i16,

    pub fire: bool,
    pub switch_weapon: bool,
}

#[derive(Resource, Default, Debug, Clone, Copy)]
pub struct PointerWorldPosition(pub Vec2);

/// Component for the weapon sprite's position relative to player
#[derive(Component, Clone, Copy, Default)]
pub struct CursorPosition {
    pub x: i32,
    pub y: i32
}


fn get_facing_direction(input: &BoxInput) -> FacingDirection {

    if input.pan_x > PAN_FACING_THRESHOLD {
        FacingDirection::Right
    } else if input.pan_x < -PAN_FACING_THRESHOLD {
        FacingDirection::Left
    } else {
        if input.buttons & INPUT_RIGHT != 0 {
            FacingDirection::Right
        } else if input.buttons & INPUT_LEFT != 0 {
            FacingDirection::Left
        } else {
            FacingDirection::Right
        }
    }
}

pub fn read_local_inputs(
    mut commands: Commands,
    players: Query<(&ActionState<PlayerAction>, &Transform, &Player), With<LocalPlayer>>,
    
    q_window: Query<&Window, With<PrimaryWindow>>,
    q_camera: Query<(&Camera, &GlobalTransform)>,
) {

    let mut local_inputs = HashMap::new();

    for (action_state, transform, player) in players.iter() {
        let mut input = BoxInput::default();

         if action_state.pressed(&PlayerAction::MoveUp) {
            input.buttons |= INPUT_UP;
         }
         if action_state.pressed(&PlayerAction::MoveDown) {
            input.buttons |= INPUT_DOWN;
         }
         if action_state.pressed(&PlayerAction::MoveLeft) {
            input.buttons |= INPUT_LEFT;
         }
         if action_state.pressed(&PlayerAction::MoveRight) {
            input.buttons |= INPUT_RIGHT;
         }

         if action_state.pressed(&PlayerAction::PointerClick) {
            input.fire = true;
         }

         if action_state.pressed(&PlayerAction::SwitchWeapon) {
            input.switch_weapon = true;
         }
         if action_state.pressed(&PlayerAction::SwitchWeaponMode) {
            input.buttons |= INPUT_SWITCH_WEAPON_MODE;
         }

         if action_state.pressed(&PlayerAction::Reload) {
            input.buttons |= INPUT_RELOAD;
         }

         if action_state.pressed(&PlayerAction::Sprint) {
            input.buttons |= INPUT_SPRINT;
         }

        if action_state.pressed(&PlayerAction::Dash) {
            input.buttons |= INPUT_DASH;
        }

        if action_state.pressed(&PlayerAction::Modifier) {
            input.buttons |= INPUT_MODIFIER;
        }


        if let Ok(window) = q_window.get_single() {
            if let Ok((camera, camera_transform)) = q_camera.get_single() {
                if let Some(cursor_position) = window.cursor_position() {
                    if let Ok(world_position) = camera.viewport_to_world_2d(camera_transform, cursor_position) {
                        let player_position = transform.translation.truncate();
                        let pointer_distance = world_position - player_position;

                        input.pan_x = (pointer_distance.x).round().clamp(i16::MIN as f32, i16::MAX as f32) as i16;
                        input.pan_y = (pointer_distance.y).round().clamp(i16::MIN as f32, i16::MAX as f32) as i16;
                    }
                }
            }
        }

        local_inputs.insert(player.handle, input);
    }

    commands.insert_resource(LocalInputs::<PeerConfig>(local_inputs));
}

pub fn apply_inputs(
    mut commands: Commands,
    inputs: Res<PlayerInputs<PeerConfig>>,
    character_configs: Res<Assets<CharacterConfig>>,
    // Query for Avian's Position if you're directly manipulating it,
    // or your own component that will store the intended move for another system.
    mut query: Query<(
        Entity,
        &WeaponInventory,
        &Transform, // Read-only for current position if needed for dash start
        &mut DashState,
        &mut FrameMovementIntent, // NEW: To store what this system decides
        &mut ActiveLayers,
        &mut FacingDirection,
        &mut CursorPosition,
        &mut SprintState,
        &CharacterConfigHandles,
        &Player,
        // Option<&mut AvianLinearVelocity> // If you decide to use a Dynamic body for some states
    ), With<Rollback>>,
) {
    for (
        entity, inventory, transform, mut dash_state, /*mut velocity,*/
        mut frame_intent, mut active_layers, mut facing_direction,
        mut cursor_position, mut sprint_state, config_handles, player,
        // opt_lin_vel
    ) in query.iter_mut() {
        if let Some(config) = character_configs.get(&config_handles.config) {
            let (input, _input_status) = inputs[player.handle];
            
            dash_state.update();
            frame_intent.delta_position = Vec2::ZERO; // Reset intent for the frame
            frame_intent.is_dashing_this_frame = false;
            frame_intent.dash_target_position = None;

            if dash_state.is_dashing {
                frame_intent.is_dashing_this_frame = true;
                let completed_fraction = 1.0 - (dash_state.dash_frames_remaining as f32 / 
                                              (config.movement.dash_duration_frames as f32));
                let dash_offset = dash_state.dash_direction * dash_state.dash_total_distance * completed_fraction;
                // Store the absolute target position for the dash this frame
                frame_intent.dash_target_position = Some(dash_state.dash_start_position + Vec3::new(dash_offset.x, dash_offset.y, 0.0));
                // No other movement if dashing
                continue;
            }
            
            // Check if player is trying to dash
            if (input.buttons & INPUT_DASH != 0) && dash_state.can_dash() {
                // Get looking direction for dash
                let look_direction = Vec2::new(input.pan_x as f32, input.pan_y as f32);

                let is_reverse_dash = (input.buttons & INPUT_MODIFIER) != 0;
                
                // If the player isn't aiming, use facing direction
                let mut dash_direction = if look_direction.length_squared() > 1.0 {
                    look_direction.normalize()
                } else {
                    Vec2::new(facing_direction.to_int() as f32, 0.0)
                };

                if is_reverse_dash {
                    dash_direction = -dash_direction;
                }
                
                dash_state.start_dash(
                    dash_direction, 
                    transform.translation, // Start dash from current Transform's translation
                    config.movement.dash_distance,
                    config.movement.dash_duration_frames
                );
                // ...
                // Signal that a dash just started, movement will be handled next frame by the is_dashing block
                frame_intent.is_dashing_this_frame = true; // Dash starts, movement applied in next frame's is_dashing block
                let dash_offset = dash_state.dash_direction * dash_state.dash_total_distance * (1.0 / config.movement.dash_duration_frames as f32);
                frame_intent.dash_target_position = Some(transform.translation + Vec3::new(dash_offset.x, dash_offset.y, 0.0));

                continue;
            }

            let is_sprinting = input.buttons & INPUT_SPRINT != 0;
            sprint_state.is_sprinting = is_sprinting;
            
            if is_sprinting {
                sprint_state.sprint_factor += config.movement.sprint_acceleration_per_frame;
                sprint_state.sprint_factor = sprint_state.sprint_factor.min(1.0);
            } else {
                sprint_state.sprint_factor -= config.movement.sprint_deceleration_per_frame;
                sprint_state.sprint_factor = sprint_state.sprint_factor.max(0.0);
            }

            let mut direction = Vec2::ZERO;
            if input.buttons & INPUT_UP != 0    { direction.y += 1.0; }
            if input.buttons & INPUT_DOWN != 0  { direction.y -= 1.0; }
            if input.buttons & INPUT_LEFT != 0  { direction.x -= 1.0; }
            if input.buttons & INPUT_RIGHT != 0 { direction.x += 1.0; }

            *facing_direction = get_facing_direction(&input);

            cursor_position.x = input.pan_x as i32;
            cursor_position.y = input.pan_y as i32;

            if direction != Vec2::ZERO {
                let sprint_multiplier = 1.0 + (config.movement.sprint_multiplier - 1.0) * sprint_state.sprint_factor;
                let move_amount = config.movement.acceleration * sprint_multiplier * FIXED_TIMESTEP; // This is now an "attempted move amount"
                // Instead of accumulating velocity, we calculate the intended delta for this frame.
                // A more physics-based approach for Dynamic bodies would be to apply forces/impulses
                // or set LinearVelocity. For Kinematic, we calculate the desired position change.
                let current_velocity_contribution = direction.normalize() * move_amount;
                frame_intent.delta_position += current_velocity_contribution;
            }
        }
    }
}

pub fn apply_friction(
    inputs: Res<PlayerInputs<PeerConfig>>,
    character_configs: Res<Assets<CharacterConfig>>,
    // This depends on how you structure the output of apply_inputs_avian
    mut query: Query<(&mut FrameMovementIntent, &CharacterConfigHandles, &Player), With<Rollback>>,
) {
    for (mut frame_intent, config_handles, player) in query.iter_mut() {
        if frame_intent.is_dashing_this_frame { continue; } // No friction during dash

        if let Some(config) = character_configs.get(&config_handles.config) {
            let (input, _input_status) = inputs[player.handle];
            let moving = input.buttons & INPUT_RIGHT != 0 || input.buttons & INPUT_LEFT != 0 || input.buttons & INPUT_UP != 0 || input.buttons & INPUT_DOWN != 0;


            if !moving && frame_intent.delta_position.length_squared() > 0.01 { // If there's intended movement but no input
                // Reduce the intended delta_position by a friction factor
                frame_intent.delta_position *= (1.0 - config.movement.friction * FIXED_TIMESTEP).max(0.0);
                if frame_intent.delta_position.length_squared() < 0.001 { // Threshold to snap to zero
                    frame_intent.delta_position = Vec2::ZERO;
                }
            }
        }
    }
}


pub fn apply_kinematic_player_movement(
    mut query: Query<(&mut Position, &FrameMovementIntent), (With<Player>)>,
) {
    for (mut avian_pos, frame_intent) in query.iter_mut() {
        if frame_intent.is_dashing_this_frame {
            if let Some(target_pos) = frame_intent.dash_target_position {
                // Directly set position for dash. Avian will resolve penetrations if any.
                avian_pos.0 = target_pos.truncate(); // Avian Position is Vec2
            }
        } else {
            // Apply the calculated delta for normal movement
            avian_pos.0 += frame_intent.delta_position;
        }
        println!("AVIAN POSITION {:?}", avian_pos);
        // After this system, Avian's physics step will run.
        // It will see the new avian_pos.0, detect collisions, and correct avian_pos.0
        // to prevent penetration. Bevy's Transform will then usually be updated from AvianPosition.
    }
}


pub fn update_animation_state(
    // Query Avian's LinearVelocity if available and reflective of movement,
    // or the FrameMovementIntent if that's a better proxy for "trying to move".
    // If LinearVelocity is not reliably populated for Kinematic,
    // you might need to store previous AvianPosition and compare to current to get actual delta.
    mut query: Query<(&FrameMovementIntent, /* Option<&AvianLinearVelocity>, */ &mut AnimationState), With<Rollback>>
) {
    for (frame_intent, /* opt_lin_vel, */ mut state) in query.iter_mut() {
        let current_state_name = state.0.clone();
        // Base animation on intended movement, or actual velocity if available
        let is_moving = frame_intent.delta_position.length_squared() > 0.001 || frame_intent.is_dashing_this_frame;
        // if let Some(lin_vel) = opt_lin_vel { is_moving = lin_vel.length_squared() > 0.5; }

        let new_state_name = if is_moving { "Run" } else { "Idle" };
        if current_state_name != new_state_name { state.0 = new_state_name.to_string(); }
    }
}