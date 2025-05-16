use bevy::{asset::AssetMetaCheck, prelude::*};
use bevy_ggrs::{prelude::*, GgrsSchedule};
use bevy_kira_audio::prelude::*;
use leafwing_input_manager::plugin::InputManagerPlugin;
use std::hash::Hash;
use bevy_common_assets::ron::RonAssetPlugin;

use animation::{character_visuals_spawn_system, set_sprite_flip, D2AnimationPlugin};
use bevy_ggrs::GgrsPlugin;

use crate::{audio::ZAudioPlugin, camera::CameraControlPlugin, character::{movement::Velocity, player::{config::PlayerConfig, control::PlayerAction, input::{apply_friction, apply_inputs, move_characters, read_local_inputs, update_animation_state, PointerWorldPosition}, jjrs::PeerConfig, Player}}, collider::{ weight::{process_push_accumulators, process_push_impacts, PushAccumulator, PushImpact, Weight}, CollisionSettings}, debug::SpriteDebugOverlayPlugin, frame::{increase_frame_system, FrameCount}, global_asset::{add_global_asset, loading_asset_system}, jjrs::{log_ggrs_events, setup_ggrs_local, start_matchbox_socket, wait_for_players, GggrsSessionConfiguration}, weapons::{bullet_rollback_collision_system, bullet_rollback_system, system_weapon_position, ui::WeaponDebugUIPlugin, weapon_inventory_system, weapon_rollback_system, weapons_config_update_system, Bullet, BulletRollbackState, WeaponInventory, WeaponModesState, WeaponState, WeaponsConfig}};

#[derive(Debug, Clone, Default, Eq, PartialEq, Hash, States)]
pub enum AppState {
    #[default]
    Loading,
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
        app.add_plugins(SpriteDebugOverlayPlugin{});

        app.add_plugins(ZAudioPlugin {});

        app.add_plugins(D2AnimationPlugin);
        app.add_plugins(WeaponDebugUIPlugin);
        app.add_plugins(CameraControlPlugin);

        app.add_plugins((
            RonAssetPlugin::<PlayerConfig>::new(&["ron"]),
            RonAssetPlugin::<WeaponsConfig>::new(&["ron"]),
        ));

        app.add_plugins(InputManagerPlugin::<PlayerAction>::default());
        app.init_resource::<PointerWorldPosition>();


        app.init_resource::<CollisionSettings>();

        app.init_state::<AppState>();


        app.set_rollback_schedule_fps(60);
        app.add_plugins(GgrsPlugin::<PeerConfig>::default())
            .rollback_resource_with_copy::<PointerWorldPosition>()
            .rollback_resource_with_copy::<FrameCount>()
            .rollback_component_with_clone::<WeaponInventory>()
            .rollback_component_with_clone::<WeaponModesState>()
            .rollback_component_with_clone::<WeaponState>()
            .rollback_component_with_clone::<Bullet>()
            .rollback_component_with_clone::<BulletRollbackState>()
            .rollback_component_with_clone::<Transform>()
            .rollback_component_with_reflect::<Velocity>()
            .rollback_component_with_reflect::<Weight>() // Register the Weight component
            .rollback_component_with_clone::<PushAccumulator>()
            .rollback_component_with_clone::<PushImpact>()
            .rollback_component_with_reflect::<Player>();

        app.add_systems(Startup, (add_global_asset));
        app.add_systems(Update, loading_asset_system.run_if(in_state(AppState::Loading)));
        

        if self.online {
            app.add_systems(Startup, start_matchbox_socket.after(add_global_asset));
            app.add_systems(Update, wait_for_players.run_if(in_state(AppState::Lobby)));
            app.add_systems(Update, log_ggrs_events.run_if(in_state(AppState::InGame)));
        } else {
            app.add_systems(OnEnter(AppState::Lobby), setup_ggrs_local.after(add_global_asset));
        }


        app.add_systems(ReadInputs, read_local_inputs);
        app.insert_resource(FrameCount { frame: 0 });
        app.add_systems(
            GgrsSchedule, (
                // HANDLE ALL PLAYERS INPUT
                apply_inputs,
                // MOVEMENT CHARACTERS
                apply_friction.after(apply_inputs),
                process_push_impacts.after(apply_friction),
                move_characters.after(process_push_impacts),
                // WEAPON
                system_weapon_position.after(move_characters),
                weapon_rollback_system.after(system_weapon_position),
                bullet_rollback_system.after(weapon_rollback_system),
                bullet_rollback_collision_system.after(bullet_rollback_system),
                process_push_accumulators.after(bullet_rollback_collision_system),
                // ANIMATION CRATE
                character_visuals_spawn_system.after(process_push_accumulators),
                set_sprite_flip.after(character_visuals_spawn_system),
                update_animation_state.after(set_sprite_flip),
                // After each frame
                increase_frame_system.after(update_animation_state)
            ));
        app.add_systems(Update, (
            weapon_inventory_system,
            weapons_config_update_system,
        ));
    }
}
