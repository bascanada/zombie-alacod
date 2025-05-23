
use animation::{create_child_sprite, AnimationBundle, SpriteSheetConfig};
use avian2d::prelude::*;
use bevy::{prelude::*};
use bevy_kira_audio::prelude::*;

use crate::{character::config::CharacterConfigHandles, collider::{ENEMY_LAYER, PLAYER_LAYER, WALL_LAYER}, global_asset::GlobalAsset, weapons::{spawn_weapon_for_player, FiringMode, Weapon, WeaponInventory, WeaponsConfig}};

use bevy_ggrs::AddRollbackCommandExtension;

use super::{config::CharacterConfig, dash::DashState, health::{ui::HealthBar, DamageAccumulator, Health}, movement::{FrameMovementIntent, SprintState}, Character};

pub fn create_character(
    commands: &mut Commands,
    global_assets: &Res<GlobalAsset>,
    character_asset: &Res<Assets<CharacterConfig>>,
    asset_server: &Res<AssetServer>,
    texture_atlas_layouts: &mut ResMut<Assets<TextureAtlasLayout>>,
    sprint_sheet_assets: &Res<Assets<SpriteSheetConfig>>,

    config_name: String,

    skin: Option<String>,
    color_health_bar: Color,
    translation: Vec3,
) -> Entity {
    let handle = global_assets.character_configs.get(&config_name).unwrap();
    let config = character_asset.get(handle).unwrap();

    let map_layers = global_assets.spritesheets.get(&config.asset_name_ref).unwrap().clone();
    let animation_handle = global_assets.animations.get(&config.asset_name_ref).unwrap().clone();
    let player_config_handle = global_assets.character_configs.get(&config.asset_name_ref).unwrap().clone();

    let starting_layer = config.skins.get(skin.as_ref().unwrap_or(&config.starting_skin)).unwrap()
        .layers.clone();

    let animation_bundle =
        AnimationBundle::new(map_layers.clone(), animation_handle.clone(),0, starting_layer.clone());

    let health: Health = config.base_health.clone().into();
    let mut entity = commands.spawn((
        Transform::from_scale(Vec3::splat(config.scale)).with_translation(translation),
        Visibility::default(),
        SpatialAudioEmitter {instances: vec![]},
        SprintState::default(),
        DashState::default(),
        RigidBody::Kinematic{}, // Key Avian component
        avian2d::prelude::Collider::rectangle(60., 120.), // Example
        FrameMovementIntent::default(),
        CollisionLayers::new(PLAYER_LAYER, WALL_LAYER | ENEMY_LAYER), 
        health,
        Character::default(),
        CharacterConfigHandles {
            config: player_config_handle.clone(),
        },
        animation_bundle,
    ));

    let entity = entity.with_children(|parent| {
        parent.spawn((
            HealthBar,
            Sprite {
                color: color_health_bar,
                custom_size: Some(Vec2::new(30.0, 3.0)),
                ..default()
            },
            Transform::from_translation(Vec3::new(0.0, 10.0, 0.1)),
        )).add_rollback();
    });

    let entity = entity.add_rollback().id();


    for k in starting_layer.keys() {
        let spritesheet_config = sprint_sheet_assets.get(map_layers.get(k).unwrap()).unwrap();
        create_child_sprite(
            commands,
            &asset_server,
            texture_atlas_layouts,
            entity.clone(), &spritesheet_config, 0);
    }

    entity


}