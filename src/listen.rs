use anyhow::{anyhow, Result};
use log::{debug, error, info, warn};
use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    thread::{self, JoinHandle},
};

use signal_hook::iterator::Signals;
use signal_hook::{consts::TERM_SIGNALS, iterator::Handle};

use crate::{app, ipc, orientation, socket, state};

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

    let final_result = run_threads(state, args.timeout, niri_socket, iio_niri_socket);
    socket::destroy_socket(&iio_niri_socket_path)?;
    final_result
}

fn run_threads(
    state: state::State,
    timeout: u64,
    niri_socket: socket::NiriSocket,
    iio_niri_socket: socket::Socket,
) -> Result<()> {
    debug!("Creating all threads...");
    let should_stop = Arc::new(AtomicBool::new(false));
    let state = Arc::new(Mutex::new(state));
    let socket_path = iio_niri_socket.get_path();

    debug!("Registering signal handler...");
    let signals_handles = handle_signals(Arc::clone(&should_stop))?;
    debug!("Signal handler registered.");

    debug!("Registering Orientation handler...");
    let orientation_handle = handle_orientation(
        Arc::clone(&state),
        timeout,
        niri_socket,
        Arc::clone(&should_stop),
        signals_handles.0.clone(),
    );
    debug!("Orientation handler registered.");

    debug!("Registering IPC handler...");
    let ipc_handle = handle_ipc(
        Arc::clone(&should_stop),
        Arc::clone(&state),
        iio_niri_socket,
        signals_handles.0.clone(),
    );
    debug!("IPC handler registered.");

    debug!("All threads running.");
    if signals_handles.1.join().is_err() {
        return Err(anyhow!("Couldn't join signal thread."));
    }
    signals_handles.0.close();
    if orientation_handle.join().is_err() {
        return Err(anyhow!("Couldn't join orientation thread."));
    }

    if should_stop.load(Ordering::Relaxed) {
        let client = ipc::Client::bind(Some(socket_path));
        client.send(String::from("wake"))?; // Used to wake the IPC thread for clean up.
    }

    if ipc_handle.join().is_err() {
        return Err(anyhow!("Couldn't join IPC thread."));
    }
    Ok(())
}

fn handle_ipc(
    should_stop: Arc<AtomicBool>,
    state: Arc<Mutex<state::State>>,
    iio_niri_socket: socket::Socket,
    signals_handle: Handle,
) -> JoinHandle<()> {
    thread::spawn(move || {
        iio_niri_socket.process(Arc::clone(&state), Arc::clone(&should_stop));
        signals_handle.close();
    })
}

fn handle_signals(should_stop: Arc<AtomicBool>) -> Result<(Handle, JoinHandle<()>)> {
    let mut signals = match Signals::new(TERM_SIGNALS) {
        Ok(s) => s,
        Err(e) => return Err(anyhow!(e)),
    };

    let handle = signals.handle();
    let thread_handle = thread::spawn(move || {
        for _ in signals.forever() {
            should_stop.store(true, Ordering::Relaxed);
            if should_stop.load(Ordering::Relaxed) {
                warn!("The application was requested to stop.");
                info!("Cleaning up threads before exiting...");
            }
        }
    });
    Ok((handle, thread_handle))
}

fn handle_orientation(
    state: Arc<Mutex<state::State>>,
    timeout: u64,
    niri_socket: socket::NiriSocket,
    should_stop: Arc<AtomicBool>,
    signals_handle: Handle,
) -> JoinHandle<()> {
    thread::spawn(move || {
        if let Err(e) = orientation::change_orientation_routine(
            state,
            timeout,
            niri_socket,
            Arc::clone(&should_stop),
        ) {
            error!("{}", e);
            should_stop.store(true, Ordering::Relaxed);
        };
        signals_handle.close();
    })
}
