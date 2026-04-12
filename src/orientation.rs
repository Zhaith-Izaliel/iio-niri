use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    time::Duration,
};

use anyhow::{anyhow, Result};
use log::{debug, info};

use niri_ipc::{socket::Socket, OutputAction, Request};

use crate::{
    accelerometer::Accelerometer,
    monitor,
    state::{State, TransformMapping},
};

/// Update the given monitor's orientation using the accelerometer orientation.
fn update_orientation(
    socket: &mut Socket,
    monitor: &str,
    acc_orientation: &str,
    matrix: &TransformMapping,
) -> Result<()> {
    let orientation = matrix.parse_orientation(acc_orientation);

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

/// Defines the thread routine to listen to orientation changes.
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
            let orientation = accelerometer.get_orientation()?;
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
                    &state.mapping,
                )?;
            }
        }
    }

    accelerometer.release()
}
