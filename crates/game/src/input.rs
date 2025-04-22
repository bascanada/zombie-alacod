
use animation::{AnimationState, CharacterAnimationHandles};
use bevy::{prelude::*, time::Time, utils::HashMap};
use leafwing_input_manager::prelude::*;
use bevy_ggrs::prelude::*;
use bevy_ggrs::LocalInputs;


use crate::character::movement::{MovementConfig, Velocity};
use crate::character::player::jjrs::BoxConfig;
use crate::character::player::{control::PlayerAction, jjrs::BoxInput, Player};

// Runs in Bevy's PreUpdate schedule, before GGRS schedule
pub fn read_local_inputs(
    mut commands: Commands,
    players: Query<(&ActionState<PlayerAction>, &Player)>,
) {
    // Create default input (no action)

    let mut local_inputs = HashMap::new();

    // Find the ActionState for the local player(s)
    // In synctest, all players are local. In P2P, you'd filter by local player handle.
    for (action_state, player) in players.iter() {
        let mut input = BoxInput::default();
         // Check which actions are pressed
         if action_state.pressed(&PlayerAction::MoveUp) {
            input.0 |= 1;
         }
         if action_state.pressed(&PlayerAction::MoveDown) {
            input.0 |= 2;
         }
         if action_state.pressed(&PlayerAction::MoveLeft) {
            input.0 |= 3;
         }
         if action_state.pressed(&PlayerAction::MoveRight) {
            input.0 |= 4;
         }
         // Add other actions...

        // Store the input for this player's handle
        // bincode serialization happens implicitly within GGRS when inputs are exchanged.
        // GGRS handles getting this input struct from the LocalInputs resource.
        local_inputs.insert(player.handle, input);
        // In Synctest, we set input for all handles based on potentially just one ActionState query.
        // In a real P2P game, you'd only query the single local player's ActionState.
    }

    commands.insert_resource(LocalInputs::<BoxConfig>(local_inputs));
     // Log input for debugging
}

// Applies movement based on GGRS inputs
pub fn apply_inputs(
    inputs: Res<PlayerInputs<BoxConfig>>, // GGRS provides inputs for all players here
    //movement_configs: Res<Assets<MovementConfig>>,
    mut query: Query<(&mut Velocity, &CharacterAnimationHandles, &Player), With<Rollback>>,
    time: Res<Time>, 
) {
    let config = MovementConfig {
        acceleration: 10.,
        max_speed: 10.0,
        friction: 10.0
    };

    for (mut velocity, config_handles, player) in query.iter_mut() {
        // Get the config for this character
        //if let Some(config) = movement_configs.get(&config_handles.movement) {
            // Get the input for this player from GGRS
            // The input here is already deserialized GgrsInput struct
            let (input, _input_status) = inputs[player.handle];

            // Calculate direction from input struct
            let mut direction = Vec2::ZERO;
            if input.0 & 1 != 0    { direction.y += 1.0; }
            if input.0 & 2 != 0  { direction.y -= 1.0; }
            if input.0 & 3 != 0  { direction.x -= 1.0; }
            if input.0 & 4 != 0 { direction.x += 1.0; }

            // Apply acceleration if there's input
            if direction != Vec2::ZERO {
                 let move_delta = direction.normalize() * config.acceleration * time.delta().as_secs_f32();
                 velocity.0 += move_delta;
                 // Clamp velocity
                 velocity.0 = velocity.0.clamp_length_max(config.max_speed);

                 println!("VELOCITY {:?}", velocity.0);
            }
        //}
    }
}

pub fn apply_friction(
    inputs: Res<PlayerInputs<BoxConfig>>,
    //movement_configs: Res<Assets<MovementConfig>>,
    mut query: Query<(&mut Velocity, &CharacterAnimationHandles, &Player), With<Rollback>>,
    time: Res<Time>, // Use GGRS Time
) {

    let config = MovementConfig {
        acceleration: 10.,
        max_speed: 10.0,
        friction: 10.0
    };

    for (mut velocity, config_handles, player) in query.iter_mut() {
        //if let Some(config) = movement_configs.get(&config_handles.movement) {
            let (input, _input_status) = inputs[player.handle];

            // Check if any movement input is active
            let moving = input.0 & 1 != 0 || input.0 & 2 != 0 || input.0 & 3 != 0 || input.0 & 4 != 0;

            // Apply friction only if velocity exists and no movement input
            if !moving && velocity.length_squared() > 0.1 {
                velocity.0 *= (1.0 - config.friction * time.delta().as_secs_f32()).max(0.0);
                if velocity.length_squared() < 1.0 {
                    velocity.0 = Vec2::ZERO;
                }
            }
        //}
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