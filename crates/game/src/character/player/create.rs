
use animation::AnimationBundle;
use bevy::{prelude::*, utils:: HashMap};
use leafwing_input_manager::{prelude::ActionState, InputManagerBundle};
use utils::bmap;

use crate::{character::movement::Velocity, global_asset::GlobalAsset, weapons::{spawn_weapon_for_player, FiringMode, Weapon, WeaponInventory, WeaponsConfig}};

use bevy_ggrs::AddRollbackCommandExtension;
use super::{config::{PlayerConfig, PlayerConfigHandles}, control::{get_input_map, PlayerAction}, input::CursorPosition, LocalPlayer, Player};

pub fn create_player(
    commands: &mut Commands,
    weapons_asset: &Res<Assets<WeaponsConfig>>,
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
        AnimationBundle::new(map_layers, animation_handle.clone(),0, starting_layer);

    let mut entity = commands.spawn((
        Transform::from_scale(Vec3::splat(6.0)).with_translation(Vec3::new(-50.0 * handle as f32, 0.0, 0.0)),
        Visibility::default(),

        Player { handle: handle },
        CursorPosition::default(),
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

    let mut inventory = WeaponInventory::default();


    if let Some(weapons_config) = weapons_asset.get(&global_assets.weapons) {
        let mut keys: Vec<&String> = weapons_config.0.keys().collect();
        keys.sort();
        for (i, k) in keys.iter().enumerate() {
            spawn_weapon_for_player(commands, global_assets, i == 0, entity, weapons_config.0.get(*k).unwrap().clone(), &mut inventory);
        }
    } else {
        println!("NO ASSET FOR WEAPONS");
    }

    commands.entity(entity)
        .insert((
            inventory,
        ));

}