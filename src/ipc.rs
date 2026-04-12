use std::{
    fs,
    io::{BufRead, BufReader, BufWriter, Write},
    os::unix::net::{UnixListener, UnixStream},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
};

use anyhow::{anyhow, Result};
use log::{debug, error};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::{
    app::{MsgArgs, MsgSubcommandArgs},
    state::{State, TransformMapping},
};

/// Returns the IIO-Niri socket default directory
fn get_iio_niri_default_socket_directory() -> String {
    match std::env::var("XDG_RUNTIME_DIR") {
        Ok(val) => val,
        Err(e) => {
            error!("Couldn't get XDG_RUNTIME_DIR:\n{}", e);
            String::from("/tmp")
        }
    }
}

/// Returns the default IIO-Niri socket path
pub fn get_iio_niri_socket_path() -> String {
    let wayland_display = std::env::var("WAYLAND_DISPLAY");
    format!(
        "{}/iio-niri.{}.socket",
        get_iio_niri_default_socket_directory(),
        match wayland_display {
            Ok(val) => val,
            Err(e) => {
                error!("Couldn't get WAYLAND_DISPLAY:\n{}", e);
                String::from("unknown")
            }
        }
    )
}

/// Defines the available IPC action
pub enum IpcAction {
    LockRotation(bool),
    ToggleLockRotation(),
    ChangeMonitor(String),
    ChangeTransform(TransformMapping),
    Ping(),
    Stop(),
    PrintState(),
}

/// Defines the IIO-Niri's socket
pub struct Socket {
    socket: UnixListener,
    path: String,
}

/// A client to communicate with IIO-Niri's Socket.
pub struct Client {
    reader: BufReader<UnixStream>,
    writer: BufWriter<UnixStream>,
}

/// The response for a given action from a socket.
#[derive(Serialize, Deserialize)]
struct Response<T> {
    status: String,
    response: T,
}

pub type NiriSocket = niri_ipc::socket::Socket;

pub type IioNiriSocket = Socket;

impl IpcAction {
    /// Represents the action as a String
    fn action_string(&self) -> &str {
        match self {
            Self::LockRotation(_) => "lock_rotation",
            Self::ToggleLockRotation() => "toggle_lock_rotation",
            Self::ChangeMonitor(_) => "change_monitor",
            Self::ChangeTransform(_) => "change_transform",
            Self::Ping() => "ping",
            Self::Stop() => "stop",
            Self::PrintState() => "print_state",
        }
    }

    /// Parse the action from a JSON object
    pub fn from_json(json: serde_json::Value) -> Result<Self> {
        if !json.is_object() {
            return Err(anyhow!("The IpcAction JSON is malformed."));
        }

        let action = match json.get("action") {
            Some(v) => v,
            None => {
                return Err(anyhow!(
                    "The IpcAction JSON doesn't contain a field `action`"
                ))
            }
        };

        let action = match action.as_str() {
            Some(s) => s,
            None => return Err(anyhow!("The `action` field is not a String.")),
        };

        let value = match json.get("arg") {
            Some(v) => v,
            None => return Err(anyhow!("The IpcAction JSON doesn't contain a field `arg`")),
        };

        match action {
            "lock_rotation" => match value.as_bool() {
                Some(b) => Ok(IpcAction::LockRotation(b)),
                None => Err(anyhow!(
                    "The `arg` for an action of type `lock_rotation`, needs to be a boolean."
                )),
            },
            "toggle_lock_rotation" => Ok(IpcAction::ToggleLockRotation()),
            "change_monitor" => match value.as_str() {
                Some(s) => Ok(IpcAction::ChangeMonitor(String::from(s))),
                None => Err(anyhow!(
                    "The `arg` for an action of type `change_monitor`, needs to be a string."
                )),
            },
            "change_transform" => {
                let mapping = match serde_json::from_value::<TransformMapping>(value.to_owned()) {
                    Ok(m) => m,
                    Err(e) => return Err(anyhow!("The `arg` for an action of type `change_transform` couldn't be serialized into a TransformMapping:\n{}", e))
                };
                Ok(IpcAction::ChangeTransform(mapping))
            }
            "ping" => Ok(IpcAction::Ping()),
            "stop" => Ok(IpcAction::Stop()),
            "print_state" => Ok(IpcAction::PrintState()),
            _ => Err(anyhow!("The action `{}` does not exist.", action)),
        }
    }

    /// Parse the action to a JSON object, used as a way to communicate with the socket.
    pub fn to_json(&self) -> serde_json::Value {
        match self {
            Self::LockRotation(r) => json!({
                "action": self.action_string(),
                "arg": r
            }),
            Self::ToggleLockRotation() => json!({
                "action": self.action_string(),
                "arg": null,
            }),
            Self::ChangeMonitor(monitor) => json!({
                "action": self.action_string(),
                "arg": monitor
            }),
            Self::ChangeTransform(transform) => json!({
                "action": self.action_string(),
                "arg": transform
            }),
            Self::Ping() => json!({
                "action": self.action_string(),
                "arg": null
            }),
            Self::Stop() => json!({
                "action": self.action_string(),
                "arg": null
            }),
            Self::PrintState() => json!({
                "action": self.action_string(),
                "arg": null
            }),
        }
    }
}

impl<T> Response<T> {
    /// Creates a new response
    pub fn new(status: String, value: T) -> Self {
        Self {
            status,
            response: value,
        }
    }

    /// Creates a new "ok" response
    pub fn new_ok(value: T) -> Self {
        Self::new(String::from("ok"), value)
    }
}

impl Response<String> {
    /// Creates a new "error" response. It is assumed the `response` field is always a string for errors.
    pub fn new_error(message: String) -> Self {
        Self::new(String::from("error"), message)
    }
}

impl Socket {
    /// Bind the socket to the given path, defaulting to `$XDG_RUNTIME_DIR/iio-niri.$WAYLAND_DISPLAY.sock` if no path is supplied.
    pub fn bind(socket_path: Option<String>) -> Result<Self> {
        let path = match socket_path {
            Some(path) => path,
            None => get_iio_niri_socket_path(),
        };

        match UnixListener::bind(path.clone()) {
            Ok(s) => Ok(Self { socket: s, path }),
            Err(e) => Err(anyhow!("Couldn't bind socket at {}: \n{}", path, e)),
        }
    }

    /// Handle client requests
    fn handle_client(
        stream: &mut UnixStream,
        state: Arc<Mutex<State>>,
        should_stop: Arc<AtomicBool>,
    ) -> Result<()> {
        let mut client = Client::bind_to_stream(stream)?;
        debug!("Reading request from client...");
        let request = client.receive()?;
        debug!("Request read: {}", request);

        let mut state = match state.lock() {
            Ok(s) => s,
            Err(_) => {
                return Err(anyhow!(
                    "Couldn't lock on state because the value is poisonned."
                ))
            }
        };

        debug!("Parsing request as JSON...");
        let json = match serde_json::from_str::<serde_json::Value>(request.as_str()) {
            Ok(v) => v,
            Err(e) => return Err(anyhow!("Couldn't parse the request as valid JSON: {}", e)),
        };
        debug!("Parsing succeeded.");

        debug!("Constructing response to the client");
        let response = match IpcAction::from_json(json) {
            Ok(a) => Self::run_ipc_action(&mut state, a, Arc::clone(&should_stop))?,
            Err(e) => match serde_json::to_string(&Response::new_error(e.to_string())) {
                Ok(r) => r,
                Err(e) => return Err(anyhow!("Couldn't parse the response to send: {}", e)),
            },
        };
        debug!("Response constructed: {}", response);
        client.send(response)
    }

    /// Run the given IPC action, modifying the state is necessary.
    fn run_ipc_action(
        state: &mut State,
        change: IpcAction,
        should_stop: Arc<AtomicBool>,
    ) -> Result<String> {
        match change {
            IpcAction::LockRotation(b) => {
                let old = state.lock_rotation;
                state.lock_rotation = b;
                debug!(
                    "Changing `state.lock_rotation`. Old value: `{}`, New value: `{}`",
                    old, b
                );
                let response = match serde_json::to_string(&Response::new_ok(old)) {
                    Ok(s) => s,
                    Err(e) => return Err(anyhow!("Couldn't parse response to the client: {}", e)),
                };
                Ok(response)
            }
            IpcAction::ToggleLockRotation() => {
                let old = state.lock_rotation;
                state.lock_rotation = !state.lock_rotation;
                debug!(
                    "Changing `state.lock_rotation`. Old value: `{}`, New value: `{}`",
                    old, state.lock_rotation
                );
                let response = match serde_json::to_string(&Response::new_ok(old)) {
                    Ok(s) => s,
                    Err(e) => return Err(anyhow!("Couldn't parse response to the client: {}", e)),
                };
                Ok(response)
            }
            IpcAction::ChangeMonitor(monitor) => {
                let old = state.monitor.clone();
                state.monitor = monitor.clone();
                debug!(
                    "Changing `state.monitor`. Old value: `{}`, New value: `{}`",
                    old, monitor
                );
                let response = match serde_json::to_string(&Response::new(String::from("ok"), old))
                {
                    Ok(s) => s,
                    Err(e) => return Err(anyhow!("Couldn't parse response to the client: {}", e)),
                };
                Ok(response)
            }
            IpcAction::ChangeTransform(mapping) => {
                let old = state.mapping.clone();
                state.mapping = mapping.clone();
                debug!(
                    "Changing `state.mapping`. Old value: `{}`, New value: `{}`",
                    old, mapping
                );
                let response = match serde_json::to_string(&Response::new_ok(old)) {
                    Ok(s) => s,
                    Err(e) => return Err(anyhow!("Couldn't parse response to the client: {}", e)),
                };
                Ok(response)
            }
            IpcAction::Ping() => {
                let response = match serde_json::to_string(&Response::new_ok(String::from("Pong!")))
                {
                    Ok(s) => s,
                    Err(e) => return Err(anyhow!("Couldn't parse response to the client: {}", e)),
                };
                Ok(response)
            }
            IpcAction::Stop() => {
                should_stop.store(true, Ordering::Relaxed);
                debug!("Stopping the listener...");
                let response =
                    match serde_json::to_string(&Response::new_ok(String::from("Stopping!"))) {
                        Ok(s) => s,
                        Err(e) => {
                            return Err(anyhow!("Couldn't parse response to the client: {}", e))
                        }
                    };
                Ok(response)
            }
            IpcAction::PrintState() => {
                let response = match serde_json::to_string(&Response::new_ok(state)) {
                    Ok(s) => s,
                    Err(e) => return Err(anyhow!("Couldn't parse response to the client: {}", e)),
                };
                Ok(response)
            }
        }
    }

    /// Process IPC requests as they come, blocking the running thread if no action is received.
    pub fn process(&self, state: Arc<Mutex<State>>, should_stop: Arc<AtomicBool>) {
        while !should_stop.load(Ordering::Relaxed) {
            match self.socket.accept() {
                Ok((mut stream, _)) => {
                    if let Err(e) = Self::handle_client(
                        &mut stream,
                        Arc::clone(&state),
                        Arc::clone(&should_stop),
                    ) {
                        error!("{}", e);
                    }
                }
                Err(err) => error!("Couldn't connect to incoming stream: \n{}", err),
            }
        }
    }

    /// Returns the socket path
    pub fn get_path(&self) -> &str {
        self.path.as_str()
    }

    /// Removes the socket from the filesystem
    pub fn destroy_socket(&self) -> Result<()> {
        debug!("Removing socket at {}", self.path);
        match fs::remove_file(&self.path) {
            Ok(()) => Ok(()),
            Err(e) => Err(anyhow!("Couldn't not destroy socket:\n{}", e)),
        }
    }
}

impl Client {
    /// Create the underlying read/write buffers using the provided UnixStream
    fn create_buffers(
        stream: &UnixStream,
    ) -> Result<(BufReader<UnixStream>, BufWriter<UnixStream>)> {
        let reader = BufReader::new(match stream.try_clone() {
            Ok(s) => s,
            Err(e) => return Err(anyhow!("Couldn't create Reader Buffer: {}", e)),
        });

        let writer = BufWriter::new(match stream.try_clone() {
            Ok(s) => s,
            Err(e) => return Err(anyhow!("Couldn't create Writer Buffer: {}", e)),
        });

        Ok((reader, writer))
    }

    /// Binds the client to the socket, returning the bound client.
    pub fn bind(socket_path: Option<String>) -> Result<Self> {
        let path = match socket_path {
            Some(path) => path,
            None => get_iio_niri_socket_path(),
        };
        let connection = match UnixStream::connect(&path) {
            Ok(conn) => conn,
            Err(e) => return Err(anyhow!("Couldn't connect to socket {}:\n{}", &path, e)),
        };
        let buffers = Self::create_buffers(&connection)?;
        Ok(Self {
            reader: buffers.0,
            writer: buffers.1,
        })
    }

    /// Bind the client to a UnixStream, returning the bound client.
    pub fn bind_to_stream(stream: &UnixStream) -> Result<Self> {
        let buffers = Self::create_buffers(stream)?;
        Ok(Self {
            reader: buffers.0,
            writer: buffers.1,
        })
    }

    /// Send a message to the client's destination
    pub fn send(&mut self, message: String) -> Result<()> {
        if let Err(e) = self.writer.write_all(format!("{}\n", message).as_bytes()) {
            return Err(anyhow!("Couldn't write message to the stream:\n{}", e));
        }
        match self.writer.flush() {
            Ok(_) => Ok(()),
            Err(e) => Err(anyhow!("Couldn't flush buffer:\n{}", e)),
        }
    }

    /// Receive a message from the client's destination
    pub fn receive(&mut self) -> Result<String> {
        let mut message = String::new();

        match self.reader.read_line(&mut message) {
            Ok(_) => Ok(String::from(message.trim())),
            Err(e) => Err(anyhow!("Couldn't read message from the stream:\n{}", e)),
        }
    }

    /// Send a correctly formatted IPC request and waits for the response
    pub fn send_ipc_request(&mut self, request: IpcAction) -> Result<String> {
        debug!("Writing request to client: {}", request.to_json());
        self.send(request.to_json().to_string())?;
        debug!("Request sent.");

        if let Err(e) = self.writer.flush() {
            return Err(anyhow!("Couldn't flush buffer:\n{}", e));
        };

        self.receive()
    }

    /// Send an IPC request using the command line arguments to construct the request.
    pub fn send_from_args(&mut self, args: MsgArgs) -> Result<()> {
        let response = (match args.command {
            MsgSubcommandArgs::LockRotation(sub_command) => {
                self.send_ipc_request(IpcAction::LockRotation(sub_command.lock_rotation))
            }
            MsgSubcommandArgs::ToggleLockRotation(_) => {
                self.send_ipc_request(IpcAction::ToggleLockRotation())
            }
            MsgSubcommandArgs::ChangeMonitor(sub_command) => {
                self.send_ipc_request(IpcAction::ChangeMonitor(sub_command.monitor))
            }
            MsgSubcommandArgs::ChangeTransform(sub_command) => {
                self.send_ipc_request(IpcAction::ChangeTransform(
                    TransformMapping::from_transform_vec(Some(sub_command.transform))?,
                ))
            }
            MsgSubcommandArgs::Ping(_) => self.send_ipc_request(IpcAction::Ping()),
            MsgSubcommandArgs::Stop(_) => self.send_ipc_request(IpcAction::Stop()),
            MsgSubcommandArgs::PrintState(_) => self.send_ipc_request(IpcAction::PrintState()),
        })?;
        println!("{}", response);
        Ok(())
    }
}
