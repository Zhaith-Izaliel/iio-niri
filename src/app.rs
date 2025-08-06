use clap::Parser;

#[derive(Parser, Clone)]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct App {
    /// The monitor to rotate depending on the accelerometer orientation. Defaults to the first monitor Niri can see.
    #[arg(short, long)]
    pub monitor: Option<String>,

    /// The number of milliseconds before timeout for a dbus request.
    #[arg(short, long, default_value_t = 5000)]
    pub timeout: u64,

    /// The path to the niri IPC socket.
    #[arg(short, long)]
    pub niri_socket: Option<String>,
}
