use anyhow::{anyhow, Result};
use dbus::blocking::{stdintf::org_freedesktop_dbus::Properties, Connection, Proxy};
use log::{debug, info};
use niri_ipc::socket::Socket;
use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
    time::Duration,
};

use crate::{app::TransformMatrix, monitor::update_orientation};

pub const INTERFACE: &str = "net.hadess.SensorProxy";
pub const PATH: &str = "/net/hadess/SensorProxy";

pub fn listen_orientation(
    conn: &Connection,
    socket: &mut Socket,
    monitor: String,
    matrix: &TransformMatrix,
    interface: &str,
    path: &str,
    timeout: u64,
) -> Result<()> {
    let proxy = Proxy::new(interface, path, Duration::from_millis(timeout), conn);

    if !has_accelerometer(interface, &proxy)? {
        return Err(anyhow!(
            "The current hardware doesn't have an accelerometer."
        ));
    }

    debug!("Claiming accelerometer...");
    claim_accelerometer(interface, &proxy)?;
    debug!("Accelerometer claimed.");

    debug!("Creating thread for sudden exits cleanup...");
    let term = Arc::new(AtomicBool::new(false));
    signal_hook::flag::register(signal_hook::consts::SIGTERM, Arc::clone(&term))?;
    debug!("Thread created.");

    info!("Listening to accelerometer changes...");
    while !term.load(Ordering::Relaxed) {
        let found_signal = conn.process(Duration::from_millis(timeout))?;
        if found_signal {
            debug!("Found accelerometer's signal!");
            debug!("Getting orientation...");
            let orientation = get_orientation(interface, &proxy)?;
            debug!("Orientation obtained.");
            update_orientation(socket, monitor.to_owned(), orientation.as_str(), matrix)?;
        }
        thread::yield_now();
    }

    debug!("Releasing accelerometer...");
    release_accelerometer(interface, &proxy)?;
    debug!("Accelerometer released.");
    info!("Exiting!");

    Ok(())
}

fn claim_accelerometer(interface: &str, proxy: &Proxy<'_, &Connection>) -> Result<()> {
    let result: Result<(), dbus::Error> = proxy.method_call(interface, "ClaimAccelerometer", ());

    match result {
        Ok(_) => Ok(()),
        Err(err) => Err(anyhow!(format!("Couldn't claim accelerometer:\n {}", err))),
    }
}

fn release_accelerometer(interface: &str, proxy: &Proxy<'_, &Connection>) -> Result<()> {
    let result: Result<(), dbus::Error> = proxy.method_call(interface, "ReleaseAccelerometer", ());

    match result {
        Ok(_) => Ok(()),
        Err(err) => Err(anyhow!(format!(
            "Couldn't release accelerometer:\n {}",
            err
        ))),
    }
}

fn has_accelerometer(interface: &str, proxy: &Proxy<'_, &Connection>) -> Result<bool> {
    match proxy.get(interface, "HasAccelerometer") {
        Ok(it) => Ok(it),
        Err(err) => Err(anyhow!(format!(
            "Couldn't find the accelerometer on the current hardware:\n {}",
            err
        ))),
    }
}

fn get_orientation(interface: &str, proxy: &Proxy<'_, &Connection>) -> Result<String> {
    let orientation: String = match proxy.get(interface, "AccelerometerOrientation") {
        Ok(it) => it,
        Err(_) => return Err(anyhow!("Couldn't get accelerometer orientation.")),
    };

    Ok(orientation)
}
