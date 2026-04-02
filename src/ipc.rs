use anyhow::{anyhow, Result};
use log::error;
use std::{
    io::{Read, Write},
    os::unix::net::{UnixListener, UnixStream},
};

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

fn get_iio_niri_socket_path() -> String {
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

pub struct IioNiriSocket {
    socket: UnixListener,
    path: String,
}

impl IioNiriSocket {
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
        // TODO: Change state.
        println!("{}", buffer);
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

pub struct IioNiriClient {
    path: String,
}

impl IioNiriClient {
    pub fn bind(socket_path: Option<String>) -> Self {
        let path = match socket_path {
            Some(path) => path,
            None => get_iio_niri_socket_path(),
        };
        Self { path }
    }

    pub fn send(&self, message: String) {
        let stream = UnixStream::connect(self.get_path());
        match stream {
            Ok(mut stream) => {
                if let Err(e) = stream.write_all(message.into_bytes().as_slice()) {
                    error!("Couldn't write message to the stream: \n {}", e)
                }
            }
            Err(e) => {
                error!(
                    "Couldn't send message to socket ({}): \n {}",
                    self.get_path(),
                    e
                )
            }
        }
    }

    pub fn get_path(&self) -> String {
        self.path.clone()
    }
}
