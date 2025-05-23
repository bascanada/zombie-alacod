use animation::SpriteSheetConfig;
use bevy::prelude::*;

use crate::{character::{config::{CharacterConfig, CharacterConfigHandles}, create::create_character, player::input::CursorPosition},  global_asset::GlobalAsset, weapons::{WeaponInventory, WeaponsConfig}};

use super::{ai::pathing::EnemyPath, Enemy};

pub fn spawn_enemy(
    enemy_type_name: String,
    position: Vec3,
    commands: &mut Commands,
    weapons_asset: &Res<Assets<WeaponsConfig>>,
    characters_asset: &Res<Assets<CharacterConfig>>,
    asset_server: &Res<AssetServer>,
    texture_atlas_layouts: &mut ResMut<Assets<TextureAtlasLayout>>,
    sprint_sheet_assets: &Res<Assets<SpriteSheetConfig>>,

    global_assets: &Res<GlobalAsset>,
) {

    let entity = create_character(
        commands, global_assets, characters_asset, asset_server, texture_atlas_layouts, sprint_sheet_assets,
        enemy_type_name, None,
        (LinearRgba::RED).into(),position
    );

    let inventory = WeaponInventory::default();

    commands.entity(entity)
        .insert((
            inventory,
            EnemyPath::default(),
            Enemy::default(),
        ));

}