use anyhow::Result;
use log::{debug, info, warn};
use niri_ipc::{socket::Socket, Transform};

use crate::{
    app::{ListenArgs, TransformMatrix},
    monitor,
};

fn parse_transform_matrix(transform: Option<Vec<Transform>>) -> TransformMatrix {
    match transform {
        Some(vec) => TransformMatrix {
            normal: vec[0],
            left_up: vec[1],
            bottom_up: vec[2],
            right_up: vec[3],
        },
        None => TransformMatrix {
            normal: Transform::Normal,
            left_up: Transform::_90,
            bottom_up: Transform::_180,
            right_up: Transform::_270,
        },
    }
}

pub struct State {
    /// Lock the rotation of the screen
    pub lock_rotation: bool,

    /// The monitor to rotate depending on the accelerometer orientation. Defaults to the first monitor Niri can see.
    pub monitor: String,

    /// Maps the accelerometer transforms (normal,left-up,bottom-up,right-up) to Niri's transforms.
    ///
    /// In some devices the accelerometer orientation doesn't match the display orientation.
    /// This option allows you to provide the mapping from your accelerometer orientation to Niri's transform
    /// Passing a value such as 90,normal,180,270 will provide the following accelerometer mapping:
    ///
    /// - normal -> 90
    /// - left-up -> normal
    /// - bottom-up -> 180
    /// - right-up -> 270
    pub transform: TransformMatrix,

    /// The number of milliseconds before timeout for a dbus request.
    pub timeout: u64,

    /// The path to the niri IPC socket.
    pub niri_socket: Socket,
}

impl State {
    pub fn from_args(args: ListenArgs) -> Result<Self> {
        debug!("Creating state...");
        let mut niri_socket = match args.niri_socket {
            Some(path) => {
                info!("Using socket at {}.", path);
                Socket::connect_to(path)?
            }
            None => {
                warn!("Using default socket.");
                Socket::connect()?
            }
        };

        let monitor = monitor::get_monitor(&mut niri_socket, args.monitor)?;
        warn!("Using monitor {}.", monitor);
        let transform = parse_transform_matrix(args.transform);
        info!("Using transformation matrix {:?}.", transform);

        debug!("State created successfully!");
        Ok(Self {
            lock_rotation: false,
            monitor,
            transform,
            timeout: args.timeout,
            niri_socket,
        })
    }
}
