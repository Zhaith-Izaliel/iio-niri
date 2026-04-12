use clap::{ArgAction, Args, Command, Parser, Subcommand};
use clap_complete::{generate, Generator, Shell};
use clap_verbosity_flag::{Verbosity, WarnLevel};
use niri_ipc::Transform;

/// Print the completion of the given command to the given shell on stdout.
pub fn print_completions<G: Generator>(generator: G, cmd: &mut Command) {
    let name = String::from(cmd.get_name());
    generate(generator, cmd, name, &mut std::io::stdout());
}

/// Defines the entry point of the Command Parser
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

/// Defines the first level of subcommands
#[derive(Subcommand)]
pub enum Commands {
    /// Listen for the accelerometer orientation
    Listen(ListenArgs),

    /// Send a command to a running instance of IIO-Niri
    ///
    /// Each request is a JSON request in the form `{"action": <action string>, "arg": <arg (can be null)>}`.
    ///
    /// Each request has a different response when it succeeds, however the error response is always the same, in the form `{"status": "error", "response": <message>}`.
    Msg(MsgArgs),

    /// Generate completions files for a given shell
    Completions(CompletionsArgs),
}

/// Defines the arguments passed to the `listen` subcommand.
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

/// Defines the arguments passed to the `msg` subcommand.
#[derive(Args)]
pub struct MsgArgs {
    #[command(subcommand)]
    pub command: MsgSubcommandArgs,
}

/// Defines the arguments passed to the `completions` subcommand.
#[derive(Args)]
pub struct CompletionsArgs {
    /// The shell to generate completions for.
    #[arg(action=ArgAction::Set)]
    pub shell: Shell,
}

/// Defines the subcommands of the `msg` subcommand.
#[derive(Subcommand)]
pub enum MsgSubcommandArgs {
    /// Change whether to lock the rotation of the screen.
    ///
    /// If the request succeeds, returns a JSON string with `status = "ok"` and `response = <old_value>`
    LockRotation(LockRotationArgs),

    /// Toggle locking rotation.
    ///
    /// If the request succeeds, returns a JSON string with `status = "ok"` and `response = <old_value>`
    ToggleLockRotation(ToggleLockRotationArgs),

    /// Change the monitor to rotate with the accelerometer orientation.
    ///
    /// If the request succeeds, returns a JSON string with `status = "ok"` and `response = <old_value>`
    ChangeMonitor(ChangeMonitorArgs),

    /// Change the transformation mapping.
    ///
    /// If the request succeeds, returns a JSON string with `status = "ok"` and `response = <old_value>`
    ChangeTransform(ChangeTransformArgs),

    /// Ping IIO-Niri to know if its listening for requests on its IPC.
    ///
    /// If the request succeeds, returns a JSON string with `status = "ok"` and `response = "Pong!"`
    Ping(PingArgs),

    /// Stop IIO-Niri.
    ///
    /// If the request succeeds, returns a JSON string with `status = "ok"` and `response = "Stopping!"`
    Stop(StopArgs),

    /// Print IIO-Niri's current state.
    ///
    /// If the request succeeds, returns a JSON string with `status = "ok"` and `response = <current state as a JSON object>`.
    PrintState(PrintStateArgs),
}

/// Defines the arguments of the subcommand `msg lock_rotation`
#[derive(Args)]
pub struct LockRotationArgs {
    /// The value to lock the rotation of the screen.
    #[arg(action=ArgAction::Set)]
    pub lock_rotation: bool,
}

/// Defines the arguments of the subcommand `msg toggle_lock_rotation`
#[derive(Args)]
pub struct ToggleLockRotationArgs;

/// Defines the arguments of the subcommand `msg ping`
#[derive(Args)]
pub struct PingArgs;

/// Defines the arguments of the subcommand `msg stop`
#[derive(Args)]
pub struct StopArgs;

/// Defines the arguments of the subcommand `msg print-state`
#[derive(Args)]
pub struct PrintStateArgs;

/// Defines the arguments of the subcommand `msg change-monitor`
#[derive(Args)]
pub struct ChangeMonitorArgs {
    /// The monitor to rotate depending on the accelerometer orientation.
    pub monitor: String,
}

/// Defines the arguments of the subcommand `msg transform`
#[derive(Args)]
pub struct ChangeTransformArgs {
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
