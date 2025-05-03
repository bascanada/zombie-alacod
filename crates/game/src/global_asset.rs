use animation::{AnimationMapConfig, SpriteSheetConfig};
use bevy::{prelude::*, utils::HashMap};
use utils::bmap;

use crate::character::player::config::PlayerConfig;

const PLAYER_SPRITESHEET_CONFIG_PATH: &str = "ZombieShooter/Sprites/Character/player_sheet.ron";
const PLAYER_SHIRT_SPRITESHEET_CONFIG_PATH: &str = "ZombieShooter/Sprites/Character/shirt_1_sheet.ron";
const PLAYER_HAIR_SPRITESHEET_CONFIG_PATH: &str = "ZombieShooter/Sprites/Character/hair_1_sheet.ron";
const PLAYER_ANIMATIONS_CONFIG_PATH: &str = "ZombieShooter/Sprites/Character/player_animation.ron";
const PLAYER_CONFIG_PATH: &str = "ZombieShooter/Sprites/Character/player_config.ron";


#[derive(Resource)]
pub struct GlobalAsset {
    pub spritesheets: HashMap<String, Handle<SpriteSheetConfig>>,
    pub animations: HashMap<String, Handle<AnimationMapConfig>>,
    pub player_configs: HashMap<String, Handle<PlayerConfig>>,
}

impl GlobalAsset {

/*
    pub fn create(asset_server: &AssetServer) -> Self {
        Self {
            spritesheets: bmap!(
                "body" => asset_server.load(PLAYER_SPRITESHEET_CONFIG_PATH),
                "shirt" => asset_server.load(PLAYER_SHIRT_SPRITESHEET_CONFIG_PATH),
                "hair" => asset_server.load(PLAYER_HAIR_SPRITESHEET_CONFIG_PATH)
            ),
            animations: bmap!("player" => asset_server.load(PLAYER_ANIMATIONS_CONFIG_PATH)),
            

        }
        let sprite_sheet_handle: Handle<SpriteSheetConfig> =
        ;
    let sprite_sheet_shirt_handle: Handle<SpriteSheetConfig> =
        asset_server.load(PLAYER_SHIRT_SPRITESHEET_CONFIG_PATH);
    let sprite_sheet_hair_handle: Handle<SpriteSheetConfig> =
        asset_server.load(PLAYER_HAIR_SPRITESHEET_CONFIG_PATH);
    let animation_handle: Handle<AnimationMapConfig> =
        asset_server.load(PLAYER_ANIMATIONS_CONFIG_PATH);
    
    let player_config_handle: Handle<PlayerConfig> = asset_server.load(PLAYER_CONFIG_PATH);
        
    }*/
    
}