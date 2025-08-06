use anyhow::{anyhow, Result};
use dbus::{
    blocking::{stdintf::org_freedesktop_dbus::Properties, Connection, Proxy},
    message::MatchRule,
    Message,
};
use std::time::Duration;

use crate::{app, monitor};

pub const INTERFACE: &str = "net.hadess.SensorProxy";
pub const PATH: &str = "/net/hadess/SensorProxy";

pub fn listen_orientation(interface: &str, path: &str, config: app::App) -> Result<()> {
    let conn = match Connection::new_system() {
        Ok(it) => it,
        Err(_) => return Err(anyhow!("Couldn't open a connection to the system bus.")),
    };

    conn.add_match_no_cb("type='signal',interface='org.freedesktop.DBus.Properties'")?;
    conn.add_match_no_cb(format!("type='signal',sender='org.freedesktop.DBus',interface='org.freedesktop.DBus',member='NameOwnerChanged',arg0='{}'", interface).as_str())?;

    let proxy = Proxy::new(
        interface,
        path,
        Duration::from_millis(config.timeout),
        &conn,
    );

    if !has_accelerometer(interface, &proxy)? {
        return Err(anyhow!(
            "The current hardware doesn't have an accelerometer."
        ));
    }

    conn.add_match(
        MatchRule::new_signal("org.freedesktop.DBus", "PropertiesChanged"),
        |_: (), _: &Connection, _: &Message| true,
    )?;

    claim_accelerometer(interface, &proxy)?;

    loop {
        let found_signal = conn.process(Duration::from_millis(config.timeout))?;
        if found_signal {
            let orientation = get_orientation(interface, &proxy)?;
            monitor::update_orientation(orientation.as_str(), config.clone())?;
        }
    }
}

fn claim_accelerometer(interface: &str, proxy: &Proxy<'_, &Connection>) -> Result<()> {
    let result: Result<(), dbus::Error> = proxy.method_call(interface, "ClaimAccelerometer", ());

    match result {
        Ok(_) => Ok(()),
        Err(err) => {
            println!("{}", err);
            Err(anyhow!("Couldn't claim accelerometer."))
        }
    }
}

fn has_accelerometer(interface: &str, proxy: &Proxy<'_, &Connection>) -> Result<bool> {
    match proxy.get(interface, "HasAccelerometer") {
        Ok(it) => Ok(it),
        Err(err) => {
            println!("{}", err);
            Err(anyhow!(
                "Couldn't find if there is an accelerometer or not."
            ))
        }
    }
}

fn get_orientation(interface: &str, proxy: &Proxy<'_, &Connection>) -> Result<String> {
    let orientation: String = match proxy.get(interface, "AccelerometerOrientation") {
        Ok(it) => it,
        Err(_) => return Err(anyhow!("Couldn't get accelerometer orientation.")),
    };

    Ok(orientation)
}
