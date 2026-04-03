use anyhow::{anyhow, Result};
use log::{debug, info, warn};
use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    thread,
    time::Duration,
};

use signal_hook::{consts::TERM_SIGNALS, iterator::Signals};

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
    let state = Arc::new(Mutex::new(state::State::from_args(
        &mut niri_socket,
        &args,
    )?));
    debug!("State created successfully!");

    debug!("Creating IIO-Niri socket...");
    let iio_niri_socket = socket::Socket::bind(iio_niri_socket_path)?;
    let iio_niri_socket_path = iio_niri_socket.get_path();
    debug!("Socket created at {}", iio_niri_socket.get_path());

    // Shouldn't return until the end.

    let should_stop = Arc::new(AtomicBool::new(false));
    debug!("Created threads stop condition.");

    debug!("Creating all threads...");
    let mut handles = Vec::with_capacity(2);

    let ipc_should_stop = Arc::clone(&should_stop);
    let ipc_state = Arc::clone(&state);
    let handle_ipc = thread::spawn(move || {
        let should_stop = Arc::clone(&ipc_should_stop);
        let result = handle_ipc(ipc_should_stop, iio_niri_socket, ipc_state);
        if result.is_err() {
            should_stop.store(true, Ordering::SeqCst);
        }
        result
    });
    handles.push(handle_ipc);

    let orientation_should_stop = Arc::clone(&should_stop);
    let orientation_state = Arc::clone(&state);
    let handle_orientation = thread::spawn(move || {
        let should_stop = Arc::clone(&orientation_should_stop);
        let result = handle_orientation(
            orientation_should_stop,
            orientation_state,
            args.timeout,
            niri_socket,
        );
        if result.is_err() {
            should_stop.store(true, Ordering::SeqCst);
        }
        result
    });
    handles.push(handle_orientation);
    debug!("All threads created...");

    let mut signals = Signals::new(TERM_SIGNALS).unwrap();
    while !should_stop.load(Ordering::SeqCst) {
        for _ in signals.forever() {
            should_stop.store(true, Ordering::SeqCst);
        }
    }
    signals.handle().close();

    let mut final_result = Ok(());
    for handle in handles {
        let result = handle.join();
        final_result = match result {
            Ok(r) => {
                if r.is_err() {
                    r
                } else {
                    Ok(())
                }
            }
            Err(_) => Err(anyhow!("Couldn't join thread.")),
        }
    }

    // Can now return.
    socket::destroy_socket(&iio_niri_socket_path)?;
    final_result
}

fn handle_ipc(
    should_stop: Arc<AtomicBool>,
    iio_niri_socket: socket::Socket,
    state: Arc<Mutex<state::State>>,
) -> Result<()> {
    while !should_stop.load(Ordering::SeqCst) {
        iio_niri_socket.process(Arc::clone(&state))?;
    }
    Ok(())
}

fn handle_orientation(
    should_stop: Arc<AtomicBool>,
    state: Arc<Mutex<state::State>>,
    timeout: u64,
    mut niri_socket: socket::NiriSocket,
) -> Result<()> {
    let dbus_connection = proxy::create_dbus_connection()?;
    let proxy = proxy::create_proxy(&dbus_connection, timeout)?;

    info!("Listening to accelerometer changes...");
    while !should_stop.load(Ordering::SeqCst) {
        let found_signal = dbus_connection.process(Duration::from_millis(timeout))?;
        if found_signal {
            debug!("Found accelerometer's signal!");

            debug!("Getting orientation...");
            let orientation = orientation::get_orientation(&proxy)?;
            debug!("Orientation obtained.");

            debug!("Locking on State...");
            let state = match state.lock() {
                Ok(s) => s,
                Err(_) => {
                    return Err(anyhow!(
                        "Couldn't lock on state because the mutex was poisonned."
                    ))
                }
            };
            if !state.lock_rotation {
                debug!("Lock acquired");
                update_orientation(
                    &mut niri_socket,
                    &state.monitor, // Should fail
                    orientation.as_str(),
                    &state.transform,
                )?;
            }
        }
    }
    Ok(())
}
