
use animation::{AnimationState, CharacterAnimationHandles};
use bevy::{prelude::*, time::Time, utils::HashMap};
use leafwing_input_manager::prelude::*;
use bevy_ggrs::prelude::*;
use bevy_ggrs::LocalInputs;

use crate::character::movement::{MovementConfig, Velocity};
use crate::character::player::jjrs::BoxConfig;
use crate::character::player::{control::PlayerAction, jjrs::BoxInput, Player};

use super::config::PlayerConfig;
use super::config::PlayerConfigHandles;

const INPUT_UP: u16 = 1 << 0;
const INPUT_DOWN: u16 = 1 << 1;
const INPUT_LEFT: u16 = 1 << 2;
const INPUT_RIGHT: u16 = 1 << 3;

pub fn read_local_inputs(
    mut commands: Commands,
    players: Query<(&ActionState<PlayerAction>, &Player)>,
) {

    let mut local_inputs = HashMap::new();

    for (action_state, player) in players.iter() {
        let mut input = BoxInput::default();

         if action_state.pressed(&PlayerAction::MoveUp) {
            input.0 |= INPUT_UP;
         }
         if action_state.pressed(&PlayerAction::MoveDown) {
            input.0 |= INPUT_DOWN;
         }
         if action_state.pressed(&PlayerAction::MoveLeft) {
            input.0 |= INPUT_LEFT;
         }
         if action_state.pressed(&PlayerAction::MoveRight) {
            input.0 |= INPUT_RIGHT;
         }

        local_inputs.insert(player.handle, input);
    }

    commands.insert_resource(LocalInputs::<BoxConfig>(local_inputs));
}

pub fn apply_inputs(
    inputs: Res<PlayerInputs<BoxConfig>>,
    player_configs: Res<Assets<PlayerConfig>>,
    mut query: Query<(&mut Velocity, &PlayerConfigHandles, &Player), With<Rollback>>,
    time: Res<Time>, 
) {

    for (mut velocity, config_handles, player) in query.iter_mut() {
        if let Some(config) = player_configs.get(&config_handles.config) {
            let (input, _input_status) = inputs[player.handle];

            let mut direction = Vec2::ZERO;
            if input.0 & INPUT_UP != 0    { direction.y += 1.0; }
            if input.0 & INPUT_DOWN != 0  { direction.y -= 1.0; }
            if input.0 & INPUT_LEFT != 0  { direction.x -= 1.0; }
            if input.0 & INPUT_RIGHT != 0 { direction.x += 1.0; }

            if direction != Vec2::ZERO {
                 let move_delta = direction.normalize() * config.movement.acceleration * time.delta().as_secs_f32();
                 velocity.0 += move_delta;
                 velocity.0 = velocity.0.clamp_length_max(config.movement.max_speed);
            }
        }
    }
}

pub fn apply_friction(
    inputs: Res<PlayerInputs<BoxConfig>>,
    movement_configs: Res<Assets<PlayerConfig>>,
    mut query: Query<(&mut Velocity, &PlayerConfigHandles, &Player), With<Rollback>>,
    time: Res<Time>,
) {


    for (mut velocity, config_handles, player) in query.iter_mut() {
        if let Some(config) = movement_configs.get(&config_handles.config) {
            let (input, _input_status) = inputs[player.handle];

            let moving = input.0 & INPUT_RIGHT != 0 || input.0 & INPUT_LEFT != 0 || input.0 & INPUT_UP != 0 || input.0 & INPUT_DOWN != 0;

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