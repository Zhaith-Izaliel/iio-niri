use std::{
    io::Read,
    os::unix::net::{UnixListener, UnixStream},
};

use anyhow::{anyhow, Result};
use log::error;

use crate::state::State;

fn get_iio_niri_socket_directory() -> String {
    match std::env::var("XDG_RUNTIME_DIR") {
        Ok(val) => val,
        Err(e) => {
            error!("Couldn't get XDG_RUNTIME_DIR:\n {}", e);
            String::from("/tmp")
        }
    }
}

pub fn get_iio_niri_socket_path() -> String {
    let wayland_display = std::env::var("WAYLAND_DISPLAY");
    format!(
        "{}/iio-niri.{}.socket",
        get_iio_niri_socket_directory(),
        match wayland_display {
            Ok(val) => val,
            Err(e) => {
                error!("Couldn't get WAYLAND_DISPLAY: \n {}", e);
                String::from("unknown")
            }
        }
    )
}

pub type NiriSocket = niri_ipc::socket::Socket;

pub struct Socket {
    socket: UnixListener,
    path: String,
}

impl Socket {
    pub fn bind(socket_path: Option<String>) -> Result<Self> {
        let path = match socket_path {
            Some(path) => path,
            None => get_iio_niri_socket_path(),
        };

        match UnixListener::bind(path.clone()) {
            Ok(s) => Ok(Self { socket: s, path }),
            Err(e) => Err(anyhow!("Couldn't bind socket at {}: \n {}", path, e)),
        }
    }

    fn handle_client(stream: &mut UnixStream, state: &mut State) -> Result<()> {
        let mut buffer = String::new();
        if let Err(e) = stream.read_to_string(&mut buffer) {
            return Err(anyhow!(
                "Couldn't read message from incoming stream: \n {}",
                e
            ));
        };
        state.update_with_message(buffer.as_str())?;
        Ok(())
    }

    pub fn process(&self, state: &mut State) {
        for stream in self.socket.incoming() {
            match stream {
                Ok(mut stream) => {
                    if let Err(e) = Self::handle_client(&mut stream, state) {
                        error!("{}", e);
                    }
                }
                Err(err) => {
                    error!("Couldn't connect to incoming stream: \n {}", err);
                }
            }
        }
    }

    pub fn get_path(&self) -> String {
        self.path.clone()
    }
}
