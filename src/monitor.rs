use anyhow::{anyhow, Result};
use niri_ipc::{socket::Socket, OutputAction, Request, Response, Transform};

use crate::app;

fn parse_orientation(orientation: &str) -> Transform {
    match orientation {
        "normal" => Transform::Normal,
        "bottom-up" => Transform::_180,
        "right-up" => Transform::_270,
        "left-up" => Transform::_90,
        _ => Transform::Normal,
    }
}

pub fn update_orientation(orientation: &str, config: app::App) -> Result<()> {
    let mut socket = match config.niri_socket {
        Some(path) => Socket::connect_to(path)?,
        None => Socket::connect()?,
    };

    let orientation = parse_orientation(orientation);

    let outputs = match socket.send(Request::Outputs)? {
        Ok(it) => match it {
            Response::Outputs(outputs) => outputs,
            _ => return Err(anyhow!("Couldn't get the outputs list from Niri.")),
        },
        Err(e) => return Err(anyhow!(e)),
    };

    let monitor = match config.monitor {
        Some(mon) => {
            if !outputs.keys().any(|key| *key == mon) {
                return Err(anyhow!(format!(
                    "The provided monitor ({}) is not connected.",
                    mon
                )));
            }
            mon
        }
        None => match outputs.keys().next() {
            Some(str) => str.to_owned(),
            None => return Err(anyhow!("Couldn't select the monitor to rotate.")),
        },
    };

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
