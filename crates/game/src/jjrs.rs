use std::net::SocketAddr;

use bevy::prelude::*;
use bevy_ggrs::{ggrs::PlayerType, prelude::*};
use bevy_matchbox::{prelude::PeerState, MatchboxSocket};
use ggrs::UdpNonBlockingSocket;

use crate::{character::player::{create::create_player, jjrs::PeerConfig}, global_asset::GlobalAsset, plugins::AppState};

pub struct GggrsConnectionConfiguration {
    pub max_player: usize,
    pub input_delay: usize,
    pub desync_interval: u32,
    pub socket: bool,
    pub udp_port: u16
}

#[derive(Resource)]
pub struct GggrsSessionConfiguration {
    pub matchbox: bool,
    pub matchbox_url: String,
    pub lobby: String,
    pub connection: GggrsConnectionConfiguration,
    pub players: Vec<String>,
}


// For local connection

pub fn setup_ggrs_local(
    mut app_state: ResMut<NextState<AppState>>,
    mut commands: Commands,
    global_assets: Res<GlobalAsset>,
    session_config: Res<GggrsSessionConfiguration>,
) {
    let mut sess_build = SessionBuilder::<PeerConfig>::new()
        .with_num_players(session_config.connection.max_player)
        .with_desync_detection_mode(ggrs::DesyncDetection::On { interval: session_config.connection.desync_interval })
        .with_input_delay(session_config.connection.input_delay);

    for (i, addr) in session_config.players.iter().enumerate() {
        let local = addr == "localhost";
        if local {
            sess_build = sess_build
                .add_player(PlayerType::Local, i)
                .expect("Failed to add player");
        } else {
            let remote_addr: SocketAddr = addr.parse().unwrap();
            //sess_build = sess_build.add_player(PlayerType::Remote(remote_addr), i).expect("Failed to add player");
        }
        create_player(&mut commands, &global_assets, local, i);
    }

    // Start a synctest session
    let sess = if session_config.connection.socket == false {
        let sess = sess_build
        .start_synctest_session()
        .expect("Failed to start synctest session");

        Session::SyncTest(sess)
    } else {
        let socket = UdpNonBlockingSocket::bind_to_port(session_config.connection.udp_port).expect(format!("Failed to bind udp to {}", session_config.connection.udp_port).as_str());
        panic!("");
        //let sess = sess_build.start_p2p_session(socket).expect("failed to start p2p session");

        //Session::P2P(sess)
    };

    // Insert the GGRS session resource
    commands.insert_resource(sess);

    app_state.set(AppState::InGame);
}



// For matchbox socket connection


pub fn start_matchbox_socket(mut commands: Commands, ggrs_config: Res<GggrsSessionConfiguration>) {
    let url = format!("{}/{}?next={}", ggrs_config.matchbox_url, ggrs_config.lobby, ggrs_config.connection.max_player);
    commands.insert_resource(MatchboxSocket::new_unreliable(url));

}

pub fn wait_for_players(
    mut app_state: ResMut<NextState<AppState>>,

    mut commands: Commands, global_assets: Res<GlobalAsset>, mut socket: ResMut<MatchboxSocket>, ggrs_config: Res<GggrsSessionConfiguration>
) {
    // regularly call update_peers to update the list of connected peers
    let Ok(peer_changes) = socket.try_update_peers() else {
        warn!("socket dropped");
        return;
    };

    // Check for new connections
    for (peer, new_state) in peer_changes {
        // you can also handle the specific dis(connections) as they occur:
        match new_state {
            PeerState::Connected => info!("peer {peer} connected"),
            PeerState::Disconnected => info!("peer {peer} disconnected"),
        }
    }
    let players = socket.players();

    let num_players = ggrs_config.connection.max_player;
    if players.len() < num_players {
        return; // wait for more players
    }

    info!("All peers have joined, going in-game");
    // TODO


    // create a GGRS P2P session
    let mut session_builder = ggrs::SessionBuilder::<PeerConfig>::new()
        .with_num_players(num_players)
        .with_max_prediction_window(12)
        .with_input_delay(ggrs_config.connection.input_delay);

    for (i, player) in players.into_iter().enumerate() {
        session_builder = session_builder
            .add_player(player, i)
            .expect("failed to add player");

        
        create_player(&mut commands, &global_assets, matches!(player, PlayerType::Local), i);
    }

    // move the channel out of the socket (required because GGRS takes ownership of it)
    let channel = socket.take_channel(0).unwrap();

    // start the GGRS session
    let ggrs_session = session_builder
        .start_p2p_session(channel)
        .expect("failed to start session");


    commands.insert_resource(bevy_ggrs::Session::P2P(ggrs_session));

    app_state.set(AppState::InGame);
}

pub fn log_ggrs_events(mut session: ResMut<bevy_ggrs::Session<PeerConfig>>) {
    match session.as_mut() {
        Session::P2P(s) => {
            //println!("GGRS_SESSION : STATE {:?} FRAME {}", s.current_state(), s.current_frame());
            for event in s.events() {
                println!("GGRS Event: {event:?}");
            }
        }
        _ => panic!("This example focuses on p2p."),
    }
}