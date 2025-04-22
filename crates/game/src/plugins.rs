use bevy::{prelude::*, time::Time, utils::HashMap};
use bevy_ggrs::{ggrs::PlayerType, prelude::*, GgrsSchedule};
use bincode; // For serializing input
use leafwing_input_manager::{plugin::InputManagerPlugin, prelude::*};
use std::hash::Hash; // Needed for GGRS state checksumming if reflecting components
use bevy_common_assets::ron::RonAssetPlugin;

use animation::{AnimationState, D2AnimationPlugin};
use bevy_ggrs::GgrsPlugin;


use crate::{character::{movement::Velocity, player::{control::PlayerAction, jjrs::BoxConfig, Player}, PlayerConfig}, input::{apply_friction, apply_inputs, move_characters, read_local_inputs, update_animation_state}, jjrs::{setup_ggrs, GggrsSessionConfiguration}};

pub struct BaseZombieGamePlugin;

impl Plugin for BaseZombieGamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(D2AnimationPlugin);

        app.add_plugins((
            RonAssetPlugin::<PlayerConfig>::new(&["ron"])
        ));

        app.add_plugins(InputManagerPlugin::<PlayerAction>::default());

        app.set_rollback_schedule_fps(60);
        app.add_plugins(GgrsPlugin::<BoxConfig>::default())
            .rollback_component_with_clone::<Transform>()
            .rollback_component_with_reflect::<Velocity>()
            .rollback_component_with_reflect::<AnimationState>()
            .rollback_component_with_reflect::<Player>();


        app.insert_resource(GggrsSessionConfiguration {input_delay: 5, max_player: 1});
        app.add_systems(Startup, setup_ggrs);
        app.add_systems(ReadInputs, read_local_inputs);
        app.add_systems(
            GgrsSchedule, (
                apply_inputs,
                apply_friction.after(apply_inputs),
                move_characters.after(apply_friction),
                update_animation_state.after(move_characters),
            ));
    }
}
