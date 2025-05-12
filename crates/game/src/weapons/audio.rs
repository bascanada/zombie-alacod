use bevy::{prelude::*, utils::HashMap};
use bevy_kira_audio::prelude::*;
use serde::{Serialize, Deserialize};

use crate::character::player::Player;


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WeaponModeAudioConfig {
    pub reloading: String,
    pub firing: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WeaponAudioConfig {
    pub modes: HashMap<String, WeaponModeAudioConfig>,
}


#[derive(Debug)]
pub enum WeaponSound {
    Reloading,
    FiringStart,
    FiringOver,
    FiringBullet
}

#[derive(Event, Debug)]
pub struct WeaponSoundEvent {
    pub sound: WeaponSound,
    pub config: WeaponModeAudioConfig,
    pub emiting_entity: Entity,
}


pub fn handle_weapon_sound_event(
    mut events: EventReader<WeaponSoundEvent>,
    audio: Res<Audio>,
    asset_server: Res<AssetServer>,

    mut q_emitter_player: Query<&mut SpatialAudioEmitter, With<Player>>
) {

    for event in events.read() {
        if let Ok(mut emitter) = q_emitter_player.get_mut(event.emiting_entity) {
            match event.sound {
                WeaponSound::FiringBullet => {

                },
                WeaponSound::FiringOver => {

                },
                WeaponSound::FiringStart => {
                    let sound = audio
                        .play(asset_server.load(event.config.firing.clone()))
                        .looped()
                        .handle();

                    emitter.instances.push(sound);
                    println!("adding starting firring sound on repeat");
                },
                WeaponSound::Reloading => {

                }
            }
        }
    }

}