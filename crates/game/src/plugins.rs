use bevy::prelude::*;
use bevy_ggrs::{prelude::*, GgrsSchedule};
use leafwing_input_manager::plugin::InputManagerPlugin;
use std::hash::Hash;
use bevy_common_assets::ron::RonAssetPlugin;

use animation::{character_visuals_spawn_system, set_sprite_flip, D2AnimationPlugin};
use bevy_ggrs::GgrsPlugin;

use crate::{audio::ZAudioPlugin, character::{movement::Velocity, player::{config::PlayerConfig, control::PlayerAction, input::{apply_friction, apply_inputs, move_characters, read_local_inputs, update_animation_state}, jjrs::BoxConfig, Player}}, frame::{increase_frame_system, FrameCount}, jjrs::{log_ggrs_events, setup_ggrs_local, start_matchbox_socket, wait_for_players, GggrsSessionConfiguration}};

#[derive(Debug, Clone, Default, Eq, PartialEq, Hash, States)]
pub enum AppState {
    #[default]
    Lobby,
    InGame,
}

pub struct BaseZombieGamePlugin { online: bool }

impl BaseZombieGamePlugin {
    pub fn new(online: bool) -> Self {
        Self { online: online }
    }
}

impl Plugin for BaseZombieGamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ZAudioPlugin {});
        app.add_plugins(D2AnimationPlugin);

        app.add_plugins((
            RonAssetPlugin::<PlayerConfig>::new(&["ron"]),
        ));

        app.add_plugins(InputManagerPlugin::<PlayerAction>::default());

        app.init_state::<AppState>();

        app.set_rollback_schedule_fps(60);
        app.add_plugins(GgrsPlugin::<BoxConfig>::default())
            .rollback_resource_with_copy::<FrameCount>()
            .rollback_component_with_clone::<Transform>()
            .rollback_component_with_reflect::<Velocity>()
            .rollback_component_with_reflect::<Player>();

        if self.online {
            app.add_systems(Startup, start_matchbox_socket);
            app.add_systems(Update, wait_for_players.run_if(in_state(AppState::Lobby)));
            app.add_systems(Update, log_ggrs_events.run_if(in_state(AppState::InGame)));
        } else {
            app.add_systems(Startup, setup_ggrs_local);
        }


        app.add_systems(ReadInputs, read_local_inputs);
        app.insert_resource(FrameCount { frame: 0 });
        app.add_systems(
            GgrsSchedule, (
                apply_inputs,
                // ANIMATION CRATE
                character_visuals_spawn_system.after(apply_inputs),
                set_sprite_flip.after(character_visuals_spawn_system),
                // ....
                apply_friction.after(set_sprite_flip),
                move_characters.after(apply_friction),
                update_animation_state.after(move_characters),
                increase_frame_system.after(update_animation_state)
            ));
    }
}
