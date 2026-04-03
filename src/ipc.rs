use anyhow::{anyhow, Result};
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
            MsgSubcommandArgs::Monitor(sub_command) => {
                self.send(format!("monitor:{}", sub_command.monitor))
            }
            MsgSubcommandArgs::Timeout(sub_command) => {
                self.send(format!("timeout:{}", sub_command.timeout))
            }
            MsgSubcommandArgs::Transform(sub_command) => self.send(format!(
                "transform:{}",
                sub_command.transform.iter().fold(String::new(), |acc, e| {
                    if acc.is_empty() {
                        format!("{:?}", e)
                    } else {
                        format!("{},{:?}", acc, e)
                    }
                })
            )),
        }
    }

    pub fn get_socket_path(&self) -> String {
        self.socket_path.clone()
    }
}
