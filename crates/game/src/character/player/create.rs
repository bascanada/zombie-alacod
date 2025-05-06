
use animation::{AnimationBundle, AnimationMapConfig, SpriteSheetConfig};
use bevy::{prelude::*, utils:: HashMap};
use leafwing_input_manager::{prelude::ActionState, InputManagerBundle};
use utils::bmap;

use crate::{character::movement::Velocity, global_asset::GlobalAsset, weapons::{spawn_weapon_for_player, Weapon, WeaponInventory, WeaponPosition}};

use bevy_ggrs::AddRollbackCommandExtension;
use super::{config::{PlayerConfig, PlayerConfigHandles}, control::{get_input_map, PlayerAction}, LocalPlayer, Player};

pub fn create_player(
    commands: &mut Commands,
    global_assets: &Res<GlobalAsset>,

    local: bool,
    handle: usize,
) {

    let map_layers = global_assets.spritesheets.get("player").unwrap().clone();
    let animation_handle = global_assets.animations.get("player").unwrap().clone();
    let player_config_handle = global_assets.player_configs.get("player").unwrap().clone();

    let starting_layer = if handle < 1 {
        bmap!("shadow" => String::new(), "body" => String::new(), "shirt" => String::new())
    } else {
        bmap!("shadow" => String::new(), "body" => String::new(), "hair" => String::new())
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
    
    let entity = entity.add_rollback().id();

    let weapon = Weapon::default();
    let mut inventory = WeaponInventory{
        active_weapon_index: 0,
        weapons: vec![]
    };


    let weapon_entity = spawn_weapon_for_player(commands, global_assets, entity, weapon, &mut inventory);

    commands.entity(entity)
        .insert((
            inventory,
            WeaponPosition::default(),
        ))
        .add_child(weapon_entity);

}