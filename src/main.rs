use clap::Parser;
use log::{error, info};

mod accelerometer;
mod app;
mod ipc;
mod listen;
mod monitor;
mod orientation;
mod socket;
mod state;

fn main() {
    let args = app::App::parse();
    env_logger::Builder::new()
        .filter_level(args.verbosity.into())
        .init();

    let response = match args.command {
        app::Commands::Listen(listen_args) => listen::run(listen_args, args.socket),
        app::Commands::Msg(msg_args) => {
            let client = ipc::Client::bind(args.socket);
            client.send_from_args(msg_args)
        }
    };

    info!("Exiting!");
    match response {
        Ok(()) => (),
        Err(e) => {
            error!("{}", e);
        }
    }
}
