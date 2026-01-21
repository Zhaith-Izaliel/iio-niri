use anyhow::{anyhow, Result};
use clap::Parser;
use dbus::{blocking::Connection, message::MatchRule, Message};
use log::{debug, error, info, warn};
use niri_ipc::socket::Socket;

use crate::app::{parse_transform_matrix, App};

mod app;
mod monitor;
mod proxy;

fn main() -> Result<()> {
    let args = app::App::parse();
    let response = run(args);
    match response {
        Ok(()) => Ok(()),
        Err(e) => {
            error!("{}", e);
            Err(e)
        }
    }
}

fn run(args: App) -> Result<()> {
    env_logger::Builder::new()
        .filter_level(args.verbosity.into())
        .init();

    let mut socket = match args.niri_socket {
        Some(path) => {
            info!("Using socket at {}.", path);
            Socket::connect_to(path)?
        }
        None => {
            warn!("Using default socket.");
            Socket::connect()?
        }
    };

    let monitor = monitor::get_monitor(&mut socket, args.monitor)?;
    warn!("Using monitor {}.", monitor);
    let matrix = parse_transform_matrix(args.transform);
    info!("Using transformation matrix {:?}.", matrix);

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
        &conn,
        &mut socket,
        monitor.to_owned(),
        &matrix,
        proxy::INTERFACE,
        proxy::PATH,
        args.timeout,
    )
}
