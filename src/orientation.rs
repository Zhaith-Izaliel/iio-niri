use crate::proxy::INTERFACE;
use anyhow::{anyhow, Result};
use dbus::blocking::{stdintf::org_freedesktop_dbus::Properties, Connection, Proxy};

pub fn claim_accelerometer(proxy: &Proxy<'_, &Connection>) -> Result<()> {
    let result: Result<(), dbus::Error> = proxy.method_call(INTERFACE, "ClaimAccelerometer", ());

    match result {
        Ok(_) => Ok(()),
        Err(err) => Err(anyhow!(format!("Couldn't claim accelerometer:\n {}", err))),
    }
}

pub fn release_accelerometer(proxy: &Proxy<'_, &Connection>) -> Result<()> {
    let result: Result<(), dbus::Error> = proxy.method_call(INTERFACE, "ReleaseAccelerometer", ());

    match result {
        Ok(_) => Ok(()),
        Err(err) => Err(anyhow!(format!(
            "Couldn't release accelerometer:\n {}",
            err
        ))),
    }
}

pub fn has_accelerometer(proxy: &Proxy<'_, &Connection>) -> Result<bool> {
    match proxy.get(INTERFACE, "HasAccelerometer") {
        Ok(it) => Ok(it),
        Err(err) => Err(anyhow!(format!(
            "Couldn't find the accelerometer on the current hardware:\n {}",
            err
        ))),
    }
}

pub fn get_orientation(proxy: &Proxy<'_, &Connection>) -> Result<String> {
    let orientation: String = match proxy.get(INTERFACE, "AccelerometerOrientation") {
        Ok(it) => it,
        Err(_) => return Err(anyhow!("Couldn't get accelerometer orientation.")),
    };

    Ok(orientation)
}
