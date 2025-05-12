use bevy::prelude::*;
use bevy_kira_audio::SpatialAudioReceiver;


pub fn setup_camera(mut commands: Commands) {
    commands.spawn((Camera2d, SpatialAudioReceiver));
}
