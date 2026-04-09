use std::fmt::Display;

use anyhow::Result;
use log::{info, warn};
use niri_ipc::{socket::Socket, Transform};
use serde::{Deserialize, Serialize};

use crate::{app::ListenArgs, monitor};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransformMapping {
    pub normal: Transform,
    pub left_up: Transform,
    pub bottom_up: Transform,
    pub right_up: Transform,
}
impl TransformMapping {
    pub fn from_transform_vec(transform: Option<Vec<Transform>>) -> TransformMapping {
        match transform {
            Some(vec) => TransformMapping {
                normal: vec[0],
                left_up: vec[1],
                bottom_up: vec[2],
                right_up: vec[3],
            },
            None => TransformMapping {
                normal: Transform::Normal,
                left_up: Transform::_90,
                bottom_up: Transform::_180,
                right_up: Transform::_270,
            },
        }
    }
}

impl Display for TransformMapping {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let string = match serde_json::to_string(self) {
            Ok(s) => s,
            Err(_) => return Err(std::fmt::Error),
        };
        write!(f, "{}", string)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
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
    pub mapping: TransformMapping,
}

impl State {
    pub fn from_args(niri_socket: &mut Socket, args: &ListenArgs) -> Result<Self> {
        let monitor = monitor::get_monitor(niri_socket, args.monitor.to_owned())?;
        warn!("Using monitor {}.", monitor);
        let transform = TransformMapping::from_transform_vec(args.transform.to_owned());
        info!("Using transformation matrix {:?}.", transform);

        Ok(Self {
            lock_rotation: false,
            monitor,
            mapping: transform,
        })
    }
}
