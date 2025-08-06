use std::collections::HashMap;

use anyhow::{anyhow, Result};
use niri_ipc::{socket::Socket, Output, OutputAction, Request, Response, Transform};

fn parse_orientation(orientation: &str) -> Transform {
    match orientation {
        "normal" => Transform::Normal,
        "bottom-up" => Transform::_180,
        "right-up" => Transform::_270,
        "left-up" => Transform::_90,
        _ => Transform::Normal,
    }
}

fn get_outputs(socket: &mut Socket) -> Result<HashMap<String, Output>> {
    match socket.send(Request::Outputs)? {
        Ok(it) => match it {
            Response::Outputs(outputs) => Ok(outputs),
            _ => Err(anyhow!("Couldn't get the outputs list from Niri.")),
        },
        Err(e) => Err(anyhow!(e)),
    }
}

pub fn get_monitor(socket: &mut Socket, config_monitor: Option<String>) -> Result<String> {
    let outputs = get_outputs(socket)?;

    match config_monitor {
        Some(mon) => {
            if !outputs.keys().any(|key| *key == mon) {
                return Err(anyhow!(format!(
                    "The provided monitor ({}) is not connected.",
                    mon
                )));
            }
            Ok(mon)
        }
        None => match outputs.keys().next() {
            Some(str) => Ok(str.to_owned()),
            None => Err(anyhow!("Couldn't select the monitor to rotate.")),
        },
    }
}

pub fn update_orientation(socket: &mut Socket, monitor: String, orientation: &str) -> Result<()> {
    let orientation = parse_orientation(orientation);

    let outputs = get_outputs(socket)?;

    let old_orientation = match outputs.get(&monitor) {
        Some(output) => {
            if let Some(logical) = output.logical {
                logical.transform
            } else {
                return Err(anyhow!(format!(
                    "Couldn't get the logical output information from the provided monitor ({}).",
                    monitor
                )));
            }
        }
        None => {
            return Err(anyhow!(format!(
                "Couldn't find the provided monitor ({}) in the list of outputs.",
                monitor
            )));
        }
    };

    if old_orientation == orientation {
        return Ok(());
    }

    if let Err(str) = socket.send(Request::Output {
        output: monitor.to_owned(),
        action: OutputAction::Transform {
            transform: orientation,
        },
    })? {
        return Err(anyhow!(str));
    };

    Ok(())
}
