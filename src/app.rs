use clap::{ArgAction, Args, Parser, Subcommand};
use clap_verbosity_flag::{Verbosity, WarnLevel};
use niri_ipc::Transform;

#[derive(Parser)]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct App {
    #[command(flatten)]
    pub verbosity: Verbosity<WarnLevel>,

    #[command(subcommand)]
    pub command: Commands,

    /// Path to the socket for controlling IIO-Niri with its own IPC
    #[arg(short, long)]
    pub socket: Option<String>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Listen for the accelerometer orientation
    Listen(ListenArgs),

    /// Send a command to a running instance of IIO-Niri
    Msg(MsgArgs),
}

#[derive(Args)]
pub struct ListenArgs {
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

#[derive(Args)]
pub struct MsgArgs {
    #[command(subcommand)]
    pub command: MsgSubcommandArgs,
}

#[derive(Subcommand)]
pub enum MsgSubcommandArgs {
    /// Lock the rotation of the screen.
    LockRotation(LockRotationArgs),

    /// Toggle locking rotation
    ToggleLockRotation(ToggleLockRotationArgs),

    /// Change the monitor to rotate with the accelerometer orientation.
    Monitor(MonitorArgs),

    /// Change the transformation matrix.
    Transform(TransformArgs),
}

#[derive(Args)]
pub struct LockRotationArgs {
    /// Lock the rotation of the screen.
    #[arg(action=ArgAction::Set)]
    pub lock_rotation: bool,
}

#[derive(Args)]
pub struct ToggleLockRotationArgs;

#[derive(Args)]
pub struct MonitorArgs {
    /// The monitor to rotate depending on the accelerometer orientation.
    pub monitor: String,
}

#[derive(Args)]
pub struct TransformArgs {
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
    #[clap(num_args = 4, verbatim_doc_comment)]
    pub transform: Vec<Transform>,
}
