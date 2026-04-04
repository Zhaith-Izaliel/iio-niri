use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    thread::{self, JoinHandle},
    time::Duration,
};

use anyhow::{anyhow, Result};
use log::{debug, error, info};

use dbus::blocking::{stdintf::org_freedesktop_dbus::Properties, Connection, Proxy};
use niri_ipc::{socket::Socket, OutputAction, Request, Transform};

use crate::{
    monitor,
    proxy::{self, INTERFACE},
    state::{State, TransformMatrix},
};

pub fn claim_accelerometer(proxy: &Proxy<'_, &Connection>) -> Result<()> {
    let result: Result<(), dbus::Error> = proxy.method_call(INTERFACE, "ClaimAccelerometer", ());

    match result {
        Ok(_) => Ok(()),
        Err(err) => Err(anyhow!("Couldn't claim accelerometer:\n {}", err)),
    }
}

pub fn has_accelerometer(proxy: &Proxy<'_, &Connection>) -> Result<bool> {
    match proxy.get(INTERFACE, "HasAccelerometer") {
        Ok(it) => Ok(it),
        Err(err) => Err(anyhow!(
            "Couldn't find the accelerometer on the current hardware:\n {}",
            err
        )),
    }
}

pub fn get_orientation(proxy: &Proxy<'_, &Connection>) -> Result<String> {
    let orientation: String = match proxy.get(INTERFACE, "AccelerometerOrientation") {
        Ok(it) => it,
        Err(_) => return Err(anyhow!("Couldn't get accelerometer orientation.")),
    };

    Ok(orientation)
}

fn parse_orientation(orientation: &str, matrix: &TransformMatrix) -> Transform {
    match orientation {
        "normal" => matrix.normal,
        "left-up" => matrix.left_up,
        "bottom-up" => matrix.bottom_up,
        "right-up" => matrix.right_up,
        _ => matrix.normal,
    }
}

fn update_orientation(
    socket: &mut Socket,
    monitor: &str,
    orientation: &str,
    matrix: &TransformMatrix,
) -> Result<()> {
    let orientation = parse_orientation(orientation, matrix);

    let outputs = monitor::get_monitors(socket)?;

    let old_orientation = match outputs.get(monitor) {
        Some(output) => {
            if let Some(logical) = output.logical {
                logical.transform
            } else {
                return Err(anyhow!(
                    "Couldn't get the logical output information from the provided monitor ({}).",
                    monitor
                ));
            }
        }
        None => {
            return Err(anyhow!(
                "Couldn't find the provided monitor ({}) in the list of outputs.",
                monitor
            ));
        }
    };

    if old_orientation == orientation {
        return Ok(());
    }

    debug!("Updating screen orientation...");
    if let Err(str) = socket.send(Request::Output {
        output: monitor.to_owned(),
        action: OutputAction::Transform {
            transform: orientation,
        },
    })? {
        return Err(anyhow!(str));
    };
    info!(
        "Updated orientation from {:?} to {:?}.",
        old_orientation, orientation
    );

    Ok(())
}

pub fn handle_orientation(
    state: Arc<Mutex<State>>,
    timeout: u64,
    mut niri_socket: Socket,
    should_stop: Arc<AtomicBool>,
) -> JoinHandle<()> {
    let thread_handle = thread::spawn(move || {
        let dbus_connection = match proxy::create_dbus_connection() {
            Ok(c) => c,
            Err(e) => return error!("{}", e),
        };
        let proxy = match proxy::create_proxy(&dbus_connection, timeout) {
            Ok(p) => p,
            Err(e) => return error!("{}", e),
        };

        info!("Listening to accelerometer changes...");
        while !should_stop.load(Ordering::Relaxed) {
            let found_signal = match dbus_connection.process(Duration::from_millis(timeout)) {
                Ok(s) => s,
                Err(e) => return error!("{}", e),
            };

            if found_signal {
                debug!("Found accelerometer's signal!");

                debug!("Getting orientation...");
                let orientation = match get_orientation(&proxy) {
                    Ok(o) => o,
                    Err(e) => return error!("{}", e),
                };
                debug!("Orientation obtained.");

                let state = match state.lock() {
                    Ok(s) => s,
                    Err(_) => {
                        return error!("Couldn't lock on state because the data is poisonned.")
                    }
                };

                if !state.lock_rotation {
                    if let Err(e) = update_orientation(
                        &mut niri_socket,
                        &state.monitor, // Should fail
                        orientation.as_str(),
                        &state.transform,
                    ) {
                        return error!("{}", e);
                    };
                }
            }
        }
    });
    thread_handle
}
