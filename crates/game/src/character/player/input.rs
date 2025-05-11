
use animation::{toggle_layer, ActiveLayers, FacingDirection};
use animation::{AnimationState, CharacterAnimationHandles};
use bevy::window::PrimaryWindow;
use bevy::{prelude::*, time::Time, utils::HashMap};
use leafwing_input_manager::prelude::*;
use bevy_ggrs::prelude::*;
use bevy_ggrs::LocalInputs;
use serde::{Serialize, Deserialize}; 

use crate::character::movement::{MovementConfig, Velocity};
use crate::character::player::{control::PlayerAction, Player};

use super::config::PlayerConfig;
use super::config::PlayerConfigHandles;
use super::jjrs::PeerConfig;
use super::LocalPlayer;


const INPUT_UP: u16 = 1 << 0;
const INPUT_DOWN: u16 = 1 << 1;
const INPUT_LEFT: u16 = 1 << 2;
const INPUT_RIGHT: u16 = 1 << 3;

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
    player_configs: Res<Assets<PlayerConfig>>,

    mut query: Query<(Entity, &mut Velocity, &mut ActiveLayers, &mut FacingDirection, &mut CursorPosition, &PlayerConfigHandles, &Player), With<Rollback>>,

    time: Res<Time>,
) {

    for (entity, mut velocity, mut active_layers, mut facing_direction , mut cursor_position, config_handles, player) in query.iter_mut() {
        if let Some(config) = player_configs.get(&config_handles.config) {
            let (input, _input_status) = inputs[player.handle];


            let mut direction = Vec2::ZERO;
            if input.buttons & INPUT_UP != 0    { direction.y += 1.0; }
            if input.buttons & INPUT_DOWN != 0  { direction.y -= 1.0; }
            if input.buttons & INPUT_LEFT != 0  { direction.x -= 1.0; }
            if input.buttons & INPUT_RIGHT != 0 { direction.x += 1.0; }

            *facing_direction = get_facing_direction(&input);

            cursor_position.x = input.pan_x as i32;
            cursor_position.y = input.pan_y as i32;

            if direction != Vec2::ZERO {
                 let move_delta = direction.normalize() * config.movement.acceleration * time.delta().as_secs_f32();
                 velocity.0 += move_delta;
                 velocity.0 = velocity.0.clamp_length_max(config.movement.max_speed);
            }
        }
    }
}

pub fn apply_friction(
    inputs: Res<PlayerInputs<PeerConfig>>,
    movement_configs: Res<Assets<PlayerConfig>>,
    mut query: Query<(&mut Velocity, &PlayerConfigHandles, &Player), With<Rollback>>,
    time: Res<Time>,
) {
    for (mut velocity, config_handles, player) in query.iter_mut() {
        if let Some(config) = movement_configs.get(&config_handles.config) {
            let (input, _input_status) = inputs[player.handle];

            let moving = input.buttons & INPUT_RIGHT != 0 || input.buttons & INPUT_LEFT != 0 || input.buttons & INPUT_UP != 0 || input.buttons & INPUT_DOWN != 0;

            if !moving && velocity.length_squared() > 0.1 {
                velocity.0 *= (1.0 - config.movement.friction * time.delta().as_secs_f32()).max(0.0);
                if velocity.length_squared() < 1.0 {
                    velocity.0 = Vec2::ZERO;
                }
            }
        }
    }
}

pub fn move_characters(
    mut query: Query<(&mut Transform, &Velocity), With<Rollback>>,
    time: Res<Time>,
) {
    for (mut transform, velocity) in query.iter_mut() {
        transform.translation.x += velocity.x * time.delta().as_secs_f32();
        transform.translation.y += velocity.y * time.delta().as_secs_f32();
    }
}

pub fn update_animation_state(mut query: Query<(&Velocity, &mut AnimationState), With<Rollback>>) {
    for (velocity, mut state) in query.iter_mut() {
        let current_state_name = state.0.clone();
        let new_state_name = if velocity.length_squared() > 0.5 { "Run" } else { "Idle" };
        if current_state_name != new_state_name { state.0 = new_state_name.to_string(); }
    }
}