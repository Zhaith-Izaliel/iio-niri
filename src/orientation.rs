use crate::{monitor, proxy::INTERFACE, state::TransformMatrix};
use anyhow::{anyhow, Result};
use dbus::blocking::{stdintf::org_freedesktop_dbus::Properties, Connection, Proxy};
use log::{debug, info};
use niri_ipc::{socket::Socket, OutputAction, Request, Transform};

pub fn claim_accelerometer(proxy: &Proxy<'_, &Connection>) -> Result<()> {
    let result: Result<(), dbus::Error> = proxy.method_call(INTERFACE, "ClaimAccelerometer", ());

    match result {
        Ok(_) => Ok(()),
        Err(err) => Err(anyhow!("Couldn't claim accelerometer:\n {}", err)),
    }
}

pub fn has_accelerometer(proxy: &Proxy<'_, &Connection>) -> Result<bool> {
    match proxy.get(INTERFACE, "HasAccelerometer") {
        Ok(it) => Ok(it),
        Err(err) => Err(anyhow!(
            "Couldn't find the accelerometer on the current hardware:\n {}",
            err
        )),
    }
}

pub fn get_orientation(proxy: &Proxy<'_, &Connection>) -> Result<String> {
    let orientation: String = match proxy.get(INTERFACE, "AccelerometerOrientation") {
        Ok(it) => it,
        Err(_) => return Err(anyhow!("Couldn't get accelerometer orientation.")),
    };

    Ok(orientation)
}

fn parse_orientation(orientation: &str, matrix: &TransformMatrix) -> Transform {
    match orientation {
        "normal" => matrix.normal,
        "left-up" => matrix.left_up,
        "bottom-up" => matrix.bottom_up,
        "right-up" => matrix.right_up,
        _ => matrix.normal,
    }
}

pub fn update_orientation(
    socket: &mut Socket,
    monitor: &str,
    orientation: &str,
    matrix: &TransformMatrix,
) -> Result<()> {
    let orientation = parse_orientation(orientation, matrix);

    let outputs = monitor::get_monitors(socket)?;

    let old_orientation = match outputs.get(monitor) {
        Some(output) => {
            if let Some(logical) = output.logical {
                logical.transform
            } else {
                return Err(anyhow!(
                    "Couldn't get the logical output information from the provided monitor ({}).",
                    monitor
                ));
            }
        }
        None => {
            return Err(anyhow!(
                "Couldn't find the provided monitor ({}) in the list of outputs.",
                monitor
            ));
        }
    };

    if old_orientation == orientation {
        return Ok(());
    }

    debug!("Updating screen orientation...");
    if let Err(str) = socket.send(Request::Output {
        output: monitor.to_owned(),
        action: OutputAction::Transform {
            transform: orientation,
        },
    })? {
        return Err(anyhow!(str));
    };
    info!(
        "Updated orientation from {:?} to {:?}.",
        old_orientation, orientation
    );

    Ok(())
}
