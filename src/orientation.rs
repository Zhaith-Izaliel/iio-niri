use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    time::Duration,
};

use anyhow::{anyhow, Result};
use log::{debug, info};

use dbus::blocking::stdintf::org_freedesktop_dbus::Properties;
use niri_ipc::{socket::Socket, OutputAction, Request, Transform};

use crate::{
    accelerometer::{Accelerometer, INTERFACE},
    monitor,
    state::{State, TransformMatrix},
};

pub fn get_orientation(accelerometer: &Accelerometer) -> Result<String> {
    let orientation: String = match accelerometer
        .proxy()
        .get(INTERFACE, "AccelerometerOrientation")
    {
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

pub fn change_orientation_routine(
    state: Arc<Mutex<State>>,
    timeout: u64,
    mut niri_socket: Socket,
    should_stop: Arc<AtomicBool>,
) -> Result<()> {
    let accelerometer = Accelerometer::new(timeout)?;

    accelerometer.claim()?;

    info!("Listening to accelerometer changes...");
    while !should_stop.load(Ordering::Relaxed) {
        let found_signal = accelerometer
            .get_dbus_connection()
            .process(Duration::from_millis(timeout))?;

        if found_signal {
            debug!("Found accelerometer's signal!");

            debug!("Getting orientation...");
            let orientation = get_orientation(&accelerometer)?;
            debug!("Orientation obtained.");

            let state = match state.lock() {
                Ok(s) => s,
                Err(_) => {
                    return Err(anyhow!(
                        "Couldn't lock on state because the data is poisonned."
                    ));
                }
            };

            if !state.lock_rotation {
                update_orientation(
                    &mut niri_socket,
                    &state.monitor, // Should fail
                    orientation.as_str(),
                    &state.transform,
                )?;
            }
        }
    }

    accelerometer.release()
}
