use std::net::SocketAddr;



#[cfg(target_arch = "wasm32")]
mod web;
#[cfg(not(target_arch = "wasm32"))]
mod cli;


pub fn get_args() -> (u16, usize, Vec<String>, Vec<SocketAddr>, String, String) {

    #[cfg(not(target_arch = "wasm32"))]
    {
        use clap::Parser;
        let args = cli::Opt::parse();

        return (
            args.local_port.unwrap_or(0),
            args.number_player.unwrap_or(0),
            args.players.unwrap_or(vec![]),
            args.spectators.unwrap_or(vec![]),
            args.matchbox.unwrap_or(String::new()),
            args.lobby.unwrap_or(String::new()),
        );
    }
    #[cfg(target_arch = "wasm32")]
    {
        use web::read_canvas_data_system;
        let args = read_canvas_data_system();
        return (
            0,
            args.number_player.unwrap_or(1),
            vec!["localhost".to_string()],
            vec![],
            args.matchbox.unwrap_or(String::new()),
            args.lobby.unwrap_or(String::new()),
        );
    }

}