
use animation::AnimationBundle;
use bevy::{prelude::*};
use bevy_kira_audio::prelude::*;

use crate::{character::{config::CharacterConfigHandles, movement::Velocity}, collider::{Collider, ColliderShape, CollisionLayer, CollisionSettings}, global_asset::GlobalAsset, weapons::{spawn_weapon_for_player, FiringMode, Weapon, WeaponInventory, WeaponsConfig}};

use bevy_ggrs::AddRollbackCommandExtension;

use super::config::CharacterConfig;

pub fn create_character(
    commands: &mut Commands,
    global_assets: &Res<GlobalAsset>,
    character_asset: &Res<Assets<CharacterConfig>>,

    config_name: String,

    skin: Option<String>,
    translation: Vec3,

    collision_layer: CollisionLayer,
) -> Entity {
    let handle = global_assets.character_configs.get(&config_name).unwrap();
    let config = character_asset.get(handle).unwrap();

    let map_layers = global_assets.spritesheets.get(&config.asset_name_ref).unwrap().clone();
    let animation_handle = global_assets.animations.get(&config.asset_name_ref).unwrap().clone();
    let player_config_handle = global_assets.character_configs.get(&config.asset_name_ref).unwrap().clone();

    let starting_layer = config.skins.get(skin.as_ref().unwrap_or(&config.starting_skin)).unwrap()
        .layers.clone();

    let animation_bundle =
        AnimationBundle::new(map_layers, animation_handle.clone(),0, starting_layer);

    let collider: Collider = (&config.collider).into();
    let mut entity = commands.spawn((
        Transform::from_scale(Vec3::splat(config.scale)).with_translation(translation),
        Visibility::default(),
        SpatialAudioEmitter {instances: vec![]},
        Velocity(Vec2::ZERO),
        collider,
        collision_layer,
        CharacterConfigHandles {
            config: player_config_handle.clone(),
        },
        animation_bundle,
    ));

    entity.add_rollback().id()
}