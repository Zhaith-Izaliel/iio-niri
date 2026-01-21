use clap::Parser;
use clap_verbosity_flag::{Verbosity, WarnLevel};
use niri_ipc::Transform;

#[derive(Debug)]
pub struct TransformMatrix {
    pub normal: Transform,
    pub left_up: Transform,
    pub bottom_up: Transform,
    pub right_up: Transform,
}

pub fn parse_transform_matrix(transform: Option<Vec<Transform>>) -> TransformMatrix {
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

#[derive(Parser, Clone)]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct App {
    #[command(flatten)]
    pub verbosity: Verbosity<WarnLevel>,

    /// The monitor to rotate depending on the accelerometer orientation. Defaults to the first monitor Niri can see.
    #[arg(short, long)]
    pub monitor: Option<String>,

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
    #[clap(
        short = 'd',
        long,
        value_delimiter = ',',
        num_args = 4,
        verbatim_doc_comment
    )]
    pub transform: Option<Vec<Transform>>,

    /// The number of milliseconds before timeout for a dbus request.
    #[arg(short, long, default_value_t = 5000)]
    pub timeout: u64,

    /// The path to the niri IPC socket.
    #[arg(short, long)]
    pub niri_socket: Option<String>,
}
