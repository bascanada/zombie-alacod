use animation::{AnimationMapConfig, SpriteSheetConfig};
use bevy::{prelude::*, utils::HashMap};
use utils::bmap;

use crate::{character::player::config::PlayerConfig, plugins::AppState, weapons::WeaponsConfig};

const PLAYER_SPRITESHEET_CONFIG_PATH: &str = "ZombieShooter/Sprites/Character/player_sheet.ron";
const PLAYER_SHIRT_SPRITESHEET_CONFIG_PATH: &str = "ZombieShooter/Sprites/Character/shirt_1_sheet.ron";
const PLAYER_HAIR_SPRITESHEET_CONFIG_PATH: &str = "ZombieShooter/Sprites/Character/hair_1_sheet.ron";
const PLAYER_ANIMATIONS_CONFIG_PATH: &str = "ZombieShooter/Sprites/Character/player_animation.ron";
const PLAYER_CONFIG_PATH: &str = "ZombieShooter/Sprites/Character/player_config.ron";


#[derive(Resource)]
pub struct GlobalAsset {
    pub spritesheets: HashMap<String, HashMap<String, Handle<SpriteSheetConfig>>>,
    pub animations: HashMap<String, Handle<AnimationMapConfig>>,
    pub player_configs: HashMap<String, Handle<PlayerConfig>>,
    pub weapons: Handle<WeaponsConfig>,
}

impl GlobalAsset {
    pub fn create(asset_server: &AssetServer) -> Self {
        Self {
            spritesheets: bmap!(
                "player" => bmap!(
                    "body" => asset_server.load(PLAYER_SPRITESHEET_CONFIG_PATH),
                    "shirt" => asset_server.load(PLAYER_SHIRT_SPRITESHEET_CONFIG_PATH),
                    "hair" => asset_server.load(PLAYER_HAIR_SPRITESHEET_CONFIG_PATH),
                    "shadow" => asset_server.load("ZombieShooter/Sprites/Character/shadow_sheet.ron")
                ),
                "weapon_1" => bmap!(
                    "body" => asset_server.load("ZombieShooter/Sprites/Character/weapon_sheet.ron")
                )
            ),
            animations: bmap!(
                "player" => asset_server.load(PLAYER_ANIMATIONS_CONFIG_PATH),
                "weapon_1" => asset_server.load(PLAYER_ANIMATIONS_CONFIG_PATH)
            ),
            player_configs: bmap!(
                "player" => asset_server.load(PLAYER_CONFIG_PATH)
            ),
            weapons: asset_server.load("ZombieShooter/Sprites/Character/weapons.ron")
        }
    }
}

pub fn add_global_asset(mut commands: Commands, asset_server: Res<AssetServer>) {
    let global_asset = GlobalAsset::create(&asset_server);

    commands.insert_resource(global_asset);
}

pub fn loading_asset_system(
    mut app_state: ResMut<NextState<AppState>>,
    global_assets: Res<GlobalAsset>,
    asset_server: Res<AssetServer>,
) {

    for (_, v) in global_assets.spritesheets.iter() {
        for (_ ,handle) in v.iter() {
            if !asset_server.load_state(handle).is_loaded() {
                return;
            }
        }
    }

    for (_, handle) in global_assets.animations.iter() {
        if !asset_server.load_state(handle).is_loaded() {
            return;
        }
    }

    for (_, handle) in global_assets.player_configs.iter() {
        if !asset_server.load_state(handle).is_loaded() {
            return;
        }
    }

    if !asset_server.load_state(&global_assets.weapons).is_loaded() {
        return;
    }

    app_state.set(AppState::Lobby);
    info!("loading of asset is done , now entering lobby");
}