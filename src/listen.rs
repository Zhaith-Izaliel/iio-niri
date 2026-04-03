use anyhow::Result;
use log::{debug, info, warn};
use std::{thread, time::Duration};

use crate::{app, monitor::update_orientation, orientation, proxy, socket, state};

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
    let state = state::State::from_args(&mut niri_socket, args)?;
    debug!("State created successfully!");

    debug!("Creating IIO-Niri socket...");
    let iio_niri_socket = socket::Socket::bind(iio_niri_socket_path)?;
    debug!("Socket created at {}", iio_niri_socket.get_path());

    listen_orientation(state.to_owned(), &mut niri_socket, &iio_niri_socket)
}

fn listen_orientation(
    mut state: state::State,
    niri_socket: &mut socket::NiriSocket,
    iio_niri_socket: &socket::Socket,
) -> Result<()> {
    let dbus_connection = proxy::create_dbus_connection()?;
    let proxy = proxy::create_proxy(&dbus_connection, state.timeout)?;

    info!("Listening to accelerometer changes...");
    loop {
        iio_niri_socket.process(&mut state);

        let found_signal = dbus_connection.process(Duration::from_millis(state.timeout))?;
        if found_signal {
            debug!("Found accelerometer's signal!");
            debug!("Getting orientation...");
            let orientation = orientation::get_orientation(&proxy)?;
            debug!("Orientation obtained.");
            update_orientation(
                niri_socket,
                &state.monitor,
                orientation.as_str(),
                &state.transform,
            )?;
        }

        thread::yield_now();
    }

    // debug!("Releasing accelerometer...");
    // release_accelerometer(interface, &proxy)?;
    // debug!("Accelerometer released.");
    // info!("Exiting!");

    // Ok(())
}
