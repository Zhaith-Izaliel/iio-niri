use anyhow::{anyhow, Result};
use dbus::{
    blocking::{Connection, Proxy},
    message::MatchRule,
    Message,
};
use log::debug;
use std::time::Duration;

use crate::orientation;

pub const INTERFACE: &str = "net.hadess.SensorProxy";
pub const PATH: &str = "/net/hadess/SensorProxy";

pub fn create_dbus_connection() -> Result<Connection> {
    debug!("Connecting to the system bus...");
    let conn = match Connection::new_system() {
        Ok(it) => it,
        Err(_) => return Err(anyhow!("Couldn't open a connection to the system bus.")),
    };
    debug!("Connected to the system bus.");

    debug!("Setting matches for iio-sensor-proxy...");
    conn.add_match_no_cb("type='signal',interface='org.freedesktop.DBus.Properties'")?;
    conn.add_match_no_cb(format!("type='signal',sender='org.freedesktop.DBus',interface='org.freedesktop.DBus',member='NameOwnerChanged',arg0='{}'", INTERFACE).as_str())?;

    conn.add_match(
        MatchRule::new_signal("org.freedesktop.DBus", "PropertiesChanged"),
        |_: (), _: &Connection, _: &Message| true,
    )?;
    debug!("Finished setting matches for iio-sensor-proxy.");

    Ok(conn)
}

pub fn create_proxy<'a>(
    dbus_connection: &'a Connection,
    timeout: u64,
) -> Result<Proxy<'a, &'a Connection>> {
    let proxy = Proxy::new(
        INTERFACE,
        PATH,
        Duration::from_millis(timeout),
        dbus_connection,
    );

    if !orientation::has_accelerometer(&proxy)? {
        return Err(anyhow!(
            "The current hardware doesn't have an accelerometer."
        ));
    }

    debug!("Claiming accelerometer...");
    orientation::claim_accelerometer(&proxy)?;
    debug!("Accelerometer claimed.");
    Ok(proxy)
}
