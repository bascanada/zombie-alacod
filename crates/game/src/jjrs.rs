use bevy::prelude::*;
use bevy_ggrs::{ggrs::PlayerType, prelude::*, GgrsSchedule};

use crate::character::player::jjrs::BoxConfig;

#[derive(Resource)]
pub struct GggrsSessionConfiguration {
    pub max_player: usize,
    pub input_delay: usize,
}

pub fn setup_ggrs(
    mut commands: Commands,
    session_config: Res<GggrsSessionConfiguration>,
) {
    // Create a GgrsSessionBuilder
    let mut sess_build = SessionBuilder::<BoxConfig>::new()
        .with_num_players(session_config.max_player)
        // Define the input delay for rollback
        .with_input_delay(session_config.input_delay);

    // Add players (local or remote) - using synctest requires local players
    for i in 0..session_config.max_player {
        sess_build = sess_build
            .add_player(PlayerType::Local, i)
            .expect("Failed to add player");
    }

    // Start a synctest session
    let sess = sess_build
        .start_synctest_session()
        .expect("Failed to start synctest session");

    // Insert the GGRS session resource
    commands.insert_resource(Session::SyncTest(sess));

}