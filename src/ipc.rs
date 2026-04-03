use anyhow::{anyhow, Result};
use niri_ipc::Transform;
use std::{io::Write, os::unix::net::UnixStream};

use crate::{
    app::{MsgArgs, MsgSubcommandArgs},
    socket::get_iio_niri_socket_path,
};

pub struct Client {
    socket_path: String,
}

impl Client {
    pub fn bind(socket_path: Option<String>) -> Self {
        let path = match socket_path {
            Some(path) => path,
            None => get_iio_niri_socket_path(),
        };
        Self { socket_path: path }
    }

    pub fn send(&self, message: String) -> Result<()> {
        let stream = UnixStream::connect(self.get_socket_path());
        match stream {
            Ok(mut stream) => {
                if let Err(e) = stream.write_all(message.into_bytes().as_slice()) {
                    return Err(anyhow!("Couldn't write message to the stream: \n {}", e));
                }
                Ok(())
            }
            Err(e) => Err(anyhow!(
                "Couldn't send message to socket ({}): \n {}",
                self.get_socket_path(),
                e
            )),
        }
    }

    pub fn send_from_args(&self, args: MsgArgs) -> Result<()> {
        match args.command {
            MsgSubcommandArgs::LockRotation(sub_command) => {
                self.send(format!("lock_rotation:{}", sub_command.lock_rotation))
            }
            MsgSubcommandArgs::ToggleLockRotation(_) => {
                self.send(String::from("toggle_lock_rotation:"))
            }
            MsgSubcommandArgs::Monitor(sub_command) => {
                self.send(format!("monitor:{}", sub_command.monitor))
            }
            MsgSubcommandArgs::Transform(sub_command) => self.send(format!(
                "transform:{}",
                sub_command.transform.iter().fold(String::new(), |acc, el| {
                    let transform_string: &str = match el {
                        Transform::Normal => "normal",
                        Transform::_90 => "90",
                        Transform::_180 => "180",
                        Transform::_270 => "270",
                        Transform::Flipped => "flipped",
                        Transform::Flipped90 => "flipped-90",
                        Transform::Flipped180 => "flipped-180",
                        Transform::Flipped270 => "flipped-270",
                    };

                    if acc.is_empty() {
                        String::from(transform_string)
                    } else {
                        format!("{},{}", acc, transform_string)
                    }
                })
            )),
        }
    }

    pub fn get_socket_path(&self) -> String {
        self.socket_path.clone()
    }
}
