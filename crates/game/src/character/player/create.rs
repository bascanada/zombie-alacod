
use animation::{ActiveLayers, AnimationBundle, AnimationMapConfig, LoadingAsset, ShadowSkin, Skin, SpriteSheetConfig};
use bevy::{prelude::*, utils:: HashMap};
use leafwing_input_manager::{prelude::ActionState, InputManagerBundle};
use utils::bmap;

use crate::character::movement::Velocity;

use bevy_ggrs::AddRollbackCommandExtension;
use super::{config::{PlayerConfig, PlayerConfigHandles}, control::{get_input_map, PlayerAction}, LocalPlayer, Player};


pub fn create_player(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,

    local: bool,
    handle: usize,
) {


    let mut map_layers = HashMap::new();
    map_layers.insert("body".to_string(), sprite_sheet_handle);
    map_layers.insert("shirt".to_string(), sprite_sheet_shirt_handle);
    map_layers.insert("hair".to_string(), sprite_sheet_hair_handle);

    let animation_bundle =
        AnimationBundle::new(map_layers, animation_handle.clone(), bmap!("body" => String::new(), "create" => String::new()));

    let mut entity = commands.spawn((
        Transform::from_scale(Vec3::splat(6.0)).with_translation(Vec3::new(-50.0 * handle as f32, 0.0, 0.0)),
        Visibility::default(),

        Player { handle: handle },
        Velocity(Vec2::ZERO),

        Skin { current: "default".to_string() },
        ShadowSkin { current: "default".to_string() },

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