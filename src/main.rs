use anyhow::{anyhow, Result};
use clap::Parser;
use dbus::{blocking::Connection, message::MatchRule, Message};
use log::{debug, error};

use crate::{
    app::Commands,
    ipc::{IioNiriClient, IioNiriSocket},
    state::State,
};

mod app;
mod ipc;
mod monitor;
mod proxy;
mod state;

fn main() -> Result<()> {
    let args = app::App::parse();
    env_logger::Builder::new()
        .filter_level(args.verbosity.into())
        .init();

    let mut state: State;

    let response = match args.command {
        Commands::Listen(listen_args) => match State::from_args(listen_args) {
            Ok(val) => {
                state = val;
                listen(&mut state, args.socket)
            }
            Err(e) => Err(e),
        },
        Commands::Msg(_msg_args) => {
            let client = IioNiriClient::bind(args.socket);
            client.send(String::from("hello world!"))?;
            Err(anyhow!("Not implemented"))
        }
    };

    match response {
        Ok(()) => Ok(()),
        Err(e) => {
            error!("{}", e);
            Err(e)
        }
    }
}

fn listen(state: &mut State, iio_niri_socket_path: Option<String>) -> Result<()> {
    debug!("Creating IIO-Niri socket...");
    let iio_niri_socket = IioNiriSocket::bind(iio_niri_socket_path)?;
    debug!("Socket created at {}", iio_niri_socket.get_path());

    debug!("Connecting to the system bus...");
    let conn = match Connection::new_system() {
        Ok(it) => it,
        Err(_) => return Err(anyhow!("Couldn't open a connection to the system bus.")),
    };
    debug!("Connected to the system bus.");

    debug!("Setting matches for iio-sensor-proxy...");
    conn.add_match_no_cb("type='signal',interface='org.freedesktop.DBus.Properties'")?;
    conn.add_match_no_cb(format!("type='signal',sender='org.freedesktop.DBus',interface='org.freedesktop.DBus',member='NameOwnerChanged',arg0='{}'", proxy::INTERFACE).as_str())?;

    conn.add_match(
        MatchRule::new_signal("org.freedesktop.DBus", "PropertiesChanged"),
        |_: (), _: &Connection, _: &Message| true,
    )?;
    debug!("Finished setting matches for iio-sensor-proxy.");

    proxy::listen_orientation(
        state,
        &conn,
        &iio_niri_socket,
        proxy::INTERFACE,
        proxy::PATH,
    )
}
