use anyhow::{anyhow, Result};
use dbus::{
    blocking::{stdintf::org_freedesktop_dbus::Properties, Connection, Proxy},
    message::MatchRule,
    Message,
};
use log::debug;
use std::time::Duration;

const INTERFACE: &str = "net.hadess.SensorProxy";
const PATH: &str = "/net/hadess/SensorProxy";

/// A structure to query the manage the Accelerometer through DBus.
pub struct Accelerometer {
    dbus_connection: Connection,
    timeout: u64,
}

impl Accelerometer {
    /// Create the DBus connection to the Accelerometer
    fn create_dbus_connection() -> Result<Connection> {
        debug!("Connecting to the system bus...");
        let conn = match Connection::new_system() {
            Ok(it) => it,
            Err(_) => return Err(anyhow!("Couldn't open a connection to the system bus.")),
        };
        debug!("Connected to the system bus.");

        debug!("Setting matches for iio-sensor-proxy...");
        conn.add_match_no_cb("type='signal',interface='org.freedesktop.DBus.Properties'")?;
        conn.add_match_no_cb(format!("type='signal',sender='org.freedesktop.DBus',interface='org.freedesktop.DBus',member='NameOwnerChanged',arg0='{}'", INTERFACE).as_str())?;

        conn.add_match(
            MatchRule::new_signal("org.freedesktop.DBus", "PropertiesChanged"),
            |_: (), _: &Connection, _: &Message| true,
        )?;
        debug!("Finished setting matches for iio-sensor-proxy.");

        Ok(conn)
    }

    /// Get the current Accelerometer's orientation
    pub fn get_orientation(&self) -> Result<String> {
        let orientation: String = match self.proxy().get(INTERFACE, "AccelerometerOrientation") {
            Ok(it) => it,
            Err(_) => return Err(anyhow!("Couldn't get accelerometer orientation.")),
        };

        Ok(orientation)
    }

    /// Queries if the hardware currently has an Accelerometer.
    fn has_accelerometer(&self) -> bool {
        matches!(self.proxy().get(INTERFACE, "HasAccelerometer"), Ok(it) if it)
    }

    /// Creates the DBus proxy to query and manage the Accelerometer.
    pub fn proxy<'a>(&'a self) -> Proxy<'a, &'a Connection> {
        Proxy::new(
            INTERFACE,
            PATH,
            Duration::from_millis(self.timeout),
            &self.dbus_connection,
        )
    }

    /// Creates a new instance of Accelerometer.
    pub fn new(timeout: u64) -> Result<Self> {
        let dbus_connection = Self::create_dbus_connection()?;
        let accelerometer = Self {
            dbus_connection,
            timeout,
        };

        if !accelerometer.has_accelerometer() {
            return Err(anyhow!(
                "Couldn't find an accelerometer on the current hardware."
            ));
        }
        Ok(accelerometer)
    }

    /// Release the Accelerometer from the Proxy.
    pub fn release(&self) -> Result<()> {
        debug!("Releasing accelerometer...");
        let result: Result<(), dbus::Error> =
            self.proxy()
                .method_call(INTERFACE, "ReleaseAccelerometer", ());

        match result {
            Ok(_) => {
                debug!("Accelerometer released.");
                Ok(())
            }
            Err(err) => Err(anyhow!(format!("Couldn't release accelerometer:\n{}", err))),
        }
    }

    /// Claim the Accelerometer from the Proxy.
    pub fn claim(&self) -> Result<()> {
        debug!("Claiming accelerometer...");
        let result: Result<(), dbus::Error> =
            self.proxy()
                .method_call(INTERFACE, "ClaimAccelerometer", ());

        match result {
            Ok(_) => {
                debug!("Accelerometer claimed.");
                Ok(())
            }
            Err(err) => Err(anyhow!("Couldn't claim accelerometer:\n{}", err)),
        }
    }

    /// Returns the DBus Connection inside this object
    pub fn get_dbus_connection(&self) -> &Connection {
        &self.dbus_connection
    }
}
