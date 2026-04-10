use anyhow::Result;
use clap::Parser;
use log::{error, info};

mod accelerometer;
mod app;
mod ipc;
mod listen;
mod monitor;
mod orientation;
mod state;

fn main() -> Result<()> {
    let args = app::App::parse();
    env_logger::Builder::new()
        .filter_level(args.verbosity.into())
        .init();

    let response = match args.command {
        app::Commands::Listen(listen_args) => listen::run(listen_args, args.socket),
        app::Commands::Msg(msg_args) => match ipc::Client::bind(args.socket) {
            Ok(mut client) => client.send_from_args(msg_args),
            Err(e) => Err(e),
        },
    };

    info!("Exiting.");
    match response {
        Ok(()) => Ok(()),
        Err(e) => {
            error!("{}", e);
            Err(e)
        }
    }
}
