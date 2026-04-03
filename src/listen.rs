use anyhow::Result;
use log::{debug, info, warn};
use std::time::Duration;

use signal_hook::{consts::TERM_SIGNALS, iterator::Signals};

use crate::{app, orientation, proxy, socket, state};

pub fn run(args: app::ListenArgs, iio_niri_socket_path: Option<String>) -> Result<()> {
    debug!("Binding Niri socket...");
    let mut niri_socket = match &args.niri_socket {
        Some(path) => {
            info!("Using Niri socket at {}.", path);
            socket::NiriSocket::connect_to(path)?
        }
        None => {
            warn!("Using default Niri socket.");
            socket::NiriSocket::connect()?
        }
    };
    debug!("Niri socket bound!");

    debug!("Creating state...");
    let state = state::State::from_args(&mut niri_socket, &args)?;
    debug!("State created successfully!");

    debug!("Creating IIO-Niri socket...");
    let iio_niri_socket = socket::Socket::bind(iio_niri_socket_path)?;
    let iio_niri_socket_path = iio_niri_socket.get_path();
    debug!("Socket created at {}", iio_niri_socket.get_path());

    let final_result = handle_orientation(state, args.timeout, niri_socket, iio_niri_socket);
    socket::destroy_socket(&iio_niri_socket_path)?;
    final_result
}

fn handle_orientation(
    mut state: state::State,
    timeout: u64,
    mut niri_socket: socket::NiriSocket,
    iio_niri_socket: socket::Socket,
) -> Result<()> {
    let dbus_connection = proxy::create_dbus_connection()?;
    let proxy = proxy::create_proxy(&dbus_connection, timeout)?;

    let mut should_stop = false;
    let mut signals = Signals::new(TERM_SIGNALS).unwrap();
    let mut orientation = orientation::get_orientation(&proxy)?;

    info!("Listening to accelerometer changes...");
    while !should_stop {
        let found_signal = dbus_connection.process(Duration::from_millis(timeout))?;

        iio_niri_socket.process(&mut state)?;

        if found_signal {
            debug!("Found accelerometer's signal!");

            debug!("Getting orientation...");
            let new_orientation = orientation::get_orientation(&proxy)?;
            debug!("Orientation obtained.");

            if !state.lock_rotation && new_orientation != orientation {
                orientation::update_orientation(
                    &mut niri_socket,
                    &state.monitor, // Should fail
                    orientation.as_str(),
                    &state.transform,
                )?;
            }
            orientation = new_orientation;
        }

        // Process signals
        for _ in signals.pending() {
            warn!("The service was requested to stop.");
            should_stop = true;
        }
    }
    signals.handle().close();
    Ok(())
}
