use crate::{
    accelerometer::Accelerometer,
    monitor,
    state::{State, TransformAction, TransformMapping},
};
use anyhow::{anyhow, Result};
use log::{debug, info};
use niri_ipc::{socket::Socket, LogicalOutput, Output, OutputAction, Request};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};

/// Update the given monitor's orientation using the accelerometer orientation.
fn update_orientation(
    socket: &mut Socket,
    monitor: &str,
    acc_orientation: &str,
    matrix: &TransformMapping,
) -> Result<()> {
    match matrix.parse_orientation(acc_orientation) {
        TransformAction::KeepPrevious => Ok(()),
        TransformAction::Set(orientation) => match monitor::get_monitors(socket)?.get(monitor) {
            Some(&Output {
                logical:
                    Some(LogicalOutput {
                        transform: old_orientation,
                        ..
                    }),
                ..
            }) if old_orientation != orientation => {
                debug!("Updating screen orientation...");
                if let Err(str) = socket.send(Request::Output {
                    output: monitor.to_owned(),
                    action: OutputAction::Transform {
                        transform: orientation,
                    },
                })? {
                    return Err(anyhow!(str));
                }
                info!("Updated orientation from {old_orientation:?} to {orientation:?}.");
                Ok(())
            }
            Some(Output { logical: None, .. }) => Err(anyhow!(
                "Couldn't get the logical output information from the provided monitor ({}).",
                monitor
            )),
            None => Err(anyhow!(
                "Couldn't find the provided monitor ({}) in the list of outputs.",
                monitor
            )),
            _ => Ok(()),
        },
    }
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
            .process(accelerometer.get_timeout())?;

        if found_signal {
            debug!("Found accelerometer's signal!");

            debug!("Getting orientation...");
            let orientation = accelerometer.get_orientation()?;
            debug!("Orientation obtained.");

            let Ok(state) = state.lock() else {
                return Err(anyhow!(
                    "Couldn't lock on state because the data is poisonned."
                ));
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
