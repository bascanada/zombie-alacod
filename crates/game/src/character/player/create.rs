
use animation::{AnimationBundle, AnimationMapConfig, SpriteSheetConfig};
use bevy::prelude::*;
use leafwing_input_manager::{prelude::ActionState, InputManagerBundle};

use crate::character::movement::Velocity;

use bevy_ggrs::AddRollbackCommandExtension;
use super::{config::{PlayerConfig, PlayerConfigHandles}, control::{get_input_map, PlayerAction}, LocalPlayer, Player};

const PLAYER_SPRITESHEET_CONFIG_PATH: &str = "ZombieShooter/Sprites/Character/player_sheet.ron";
const PLAYER_ANIMATIONS_CONFIG_PATH: &str = "ZombieShooter/Sprites/Character/player_animation.ron";
const PLAYER_CONFIG_PATH: &str = "ZombieShooter/Sprites/Character/player_config.ron";




pub fn create_player(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,

    local: bool,
    handle: usize,
) {
    let sprite_sheet_handle: Handle<SpriteSheetConfig> =
        asset_server.load(PLAYER_SPRITESHEET_CONFIG_PATH);
    let animation_handle: Handle<AnimationMapConfig> =
        asset_server.load(PLAYER_ANIMATIONS_CONFIG_PATH);
    let player_config_handle: Handle<PlayerConfig> = asset_server.load(PLAYER_CONFIG_PATH);

        let animation_bundle =
            AnimationBundle::new(sprite_sheet_handle.clone(), animation_handle.clone());

        let mut entity = commands.spawn((
            Transform::from_scale(Vec3::splat(6.0)).with_translation(Vec3::new(-50.0 * handle as f32, 0.0, 0.0)),

            Player { handle: handle },
            Velocity(Vec2::ZERO),

            PlayerConfigHandles {
                config: player_config_handle.clone(),
            },

            animation_bundle,
        ));

        if local {
            entity.insert(LocalPlayer{});
            entity.insert(InputManagerBundle::<PlayerAction> {
                action_state: ActionState::default(),
                input_map: get_input_map(),
            });

            info!("Adding local player with input {}", handle);
        }

        entity.add_rollback();
}