
use animation::{AnimationBundle, AnimationMapConfig, SpriteSheetConfig};
use bevy::{prelude::*, utils:: HashMap};
use leafwing_input_manager::{prelude::ActionState, InputManagerBundle};
use utils::bmap;

use crate::{character::movement::Velocity, global_asset::GlobalAsset};

use bevy_ggrs::AddRollbackCommandExtension;
use super::{config::{PlayerConfig, PlayerConfigHandles}, control::{get_input_map, PlayerAction}, LocalPlayer, Player};


/*
const PLAYER_SPRITESHEET_CONFIG_PATH: &str = "ZombieShooter/Sprites/Character/player_sheet.ron";
const PLAYER_SHIRT_SPRITESHEET_CONFIG_PATH: &str = "ZombieShooter/Sprites/Character/shirt_1_sheet.ron";
const PLAYER_HAIR_SPRITESHEET_CONFIG_PATH: &str = "ZombieShooter/Sprites/Character/hair_1_sheet.ron";
const PLAYER_ANIMATIONS_CONFIG_PATH: &str = "ZombieShooter/Sprites/Character/player_animation.ron";
const PLAYER_CONFIG_PATH: &str = "ZombieShooter/Sprites/Character/player_config.ron";
*/

pub fn create_player(
    commands: &mut Commands,
    global_assets: &Res<GlobalAsset>,

    local: bool,
    handle: usize,
) {

    /*
    let sprite_sheet_handle: Handle<SpriteSheetConfig> = asset_server.load(PLAYER_SPRITESHEET_CONFIG_PATH);
    let sprite_sheet_shirt_handle: Handle<SpriteSheetConfig> =
        asset_server.load(PLAYER_SHIRT_SPRITESHEET_CONFIG_PATH);
    let sprite_sheet_hair_handle: Handle<SpriteSheetConfig> =
        asset_server.load(PLAYER_HAIR_SPRITESHEET_CONFIG_PATH);
    let animation_handle: Handle<AnimationMapConfig> =
        asset_server.load(PLAYER_ANIMATIONS_CONFIG_PATH);
    
    let player_config_handle: Handle<PlayerConfig> = asset_server.load(PLAYER_CONFIG_PATH);
    */


    /*
    let mut map_layers = HashMap::new();
    map_layers.insert("body".to_string(), sprite_sheet_handle);
    map_layers.insert("shirt".to_string(), sprite_sheet_shirt_handle);
    map_layers.insert("hair".to_string(), sprite_sheet_hair_handle);
    */

    let map_layers = global_assets.spritesheets.get("player").unwrap().clone();
    let animation_handle = global_assets.animations.get("player").unwrap().clone();
    let player_config_handle = global_assets.player_configs.get("player").unwrap().clone();

    let starting_layer = if handle < 1 {
        bmap!("body" => String::new(), "shirt" => String::new())
    } else {
        bmap!("body" => String::new(), "hair" => String::new())
    };

    let animation_bundle =
        AnimationBundle::new(map_layers, animation_handle.clone(), starting_layer);

    let mut entity = commands.spawn((
        Transform::from_scale(Vec3::splat(6.0)).with_translation(Vec3::new(-50.0 * handle as f32, 0.0, 0.0)),
        Visibility::default(),

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