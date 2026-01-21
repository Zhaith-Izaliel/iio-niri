use anyhow::{anyhow, Result};
use clap::Parser;
use dbus::{blocking::Connection, message::MatchRule, Message};
use niri_ipc::socket::Socket;

use crate::app::parse_transform_matrix;

mod app;
mod monitor;
mod proxy;

fn main() -> Result<()> {
    let config = app::App::parse();

    let mut socket = match config.niri_socket {
        Some(path) => Socket::connect_to(path)?,
        None => Socket::connect()?,
    };

    let monitor = monitor::get_monitor(&mut socket, config.monitor)?;
    let matrix = parse_transform_matrix(config.transform);

    let conn = match Connection::new_system() {
        Ok(it) => it,
        Err(_) => return Err(anyhow!("Couldn't open a connection to the system bus.")),
    };

    conn.add_match_no_cb("type='signal',interface='org.freedesktop.DBus.Properties'")?;
    conn.add_match_no_cb(format!("type='signal',sender='org.freedesktop.DBus',interface='org.freedesktop.DBus',member='NameOwnerChanged',arg0='{}'", proxy::INTERFACE).as_str())?;

    conn.add_match(
        MatchRule::new_signal("org.freedesktop.DBus", "PropertiesChanged"),
        |_: (), _: &Connection, _: &Message| true,
    )?;

    proxy::listen_orientation(
        &conn,
        &mut socket,
        monitor.to_owned(),
        &matrix,
        proxy::INTERFACE,
        proxy::PATH,
        config.timeout,
    )
}
