use std::collections::HashMap;

use anyhow::{anyhow, Result};
use niri_ipc::{socket::Socket, Output, Request, Response};

pub fn get_monitors(socket: &mut Socket) -> Result<HashMap<String, Output>> {
    match socket.send(Request::Outputs)? {
        Ok(it) => match it {
            Response::Outputs(outputs) => Ok(outputs),
            _ => Err(anyhow!("Couldn't get the outputs list from Niri.")),
        },
        Err(e) => Err(anyhow!(e)),
    }
}

pub fn get_monitor(socket: &mut Socket, arg_monitor: Option<String>) -> Result<String> {
    let outputs = get_monitors(socket)?;

    match arg_monitor {
        Some(mon) => {
            if !outputs.keys().any(|key| *key == mon) {
                return Err(anyhow!("The provided monitor ({}) is not connected.", mon));
            }
            Ok(mon.to_owned())
        }
        None => match outputs.keys().next() {
            Some(str) => Ok(str.to_owned()),
            None => Err(anyhow!("Couldn't select the monitor to rotate.")),
        },
    }
}
