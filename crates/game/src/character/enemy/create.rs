
use animation::AnimationBundle;
use bevy::prelude::*;
use bevy_ggrs::AddRollbackCommandExtension;
use bevy_kira_audio::SpatialAudioEmitter;
use utils::bmap;

use crate::{character::{config::CharacterConfigHandles, movement::Velocity, player::input::CursorPosition}, collider::{Collider, ColliderShape, CollisionLayer, CollisionSettings}, global_asset::GlobalAsset, weapons::{WeaponInventory, WeaponsConfig}};

pub fn spawn_enemy(
    enemy_type_name: String,
    position: Vec3,
    commands: &mut Commands,
    weapons_asset: &Res<Assets<WeaponsConfig>>,
    global_assets: &Res<GlobalAsset>,
    collision_settings: &Res<CollisionSettings>,
) {

    let map_layers = global_assets.spritesheets.get(&enemy_type_name).unwrap().clone();
    let animation_handle = global_assets.animations.get(&enemy_type_name).unwrap().clone();
    let player_config_handle = global_assets.character_configs.get(&enemy_type_name).unwrap().clone();

    let starting_layer = bmap!("bod" => String::new());

    let animation_bundle =
        AnimationBundle::new(map_layers, animation_handle.clone(),0, starting_layer);

    let mut entity = commands.spawn((
        Transform::from_scale(Vec3::splat(6.0)).with_translation(position),
        Visibility::default(),
        SpatialAudioEmitter {instances: vec![]},
        Velocity(Vec2::ZERO),
        Collider {
            offset: Vec2::new(0., -20.),
            shape: ColliderShape::Rectangle { width: 60., height: 120. },
        },
        CollisionLayer(collision_settings.enemy_layer),

        CharacterConfigHandles {
            config: player_config_handle.clone(),
        },
        animation_bundle,
    ));

    let entity = entity.add_rollback().id();

    let inventory = WeaponInventory::default();

    commands.entity(entity)
        .insert((
            inventory,
        ));

}