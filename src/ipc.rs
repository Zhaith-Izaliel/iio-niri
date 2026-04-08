use std::{
    fs,
    io::{ErrorKind, Read, Write},
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

fn get_iio_niri_socket_directory() -> String {
    match std::env::var("XDG_RUNTIME_DIR") {
        Ok(val) => val,
        Err(e) => {
            error!("Couldn't get XDG_RUNTIME_DIR:\n {}", e);
            String::from("/tmp")
        }
    }
}

pub fn get_iio_niri_socket_path() -> String {
    let wayland_display = std::env::var("WAYLAND_DISPLAY");
    format!(
        "{}/iio-niri.{}.socket",
        get_iio_niri_socket_directory(),
        match wayland_display {
            Ok(val) => val,
            Err(e) => {
                error!("Couldn't get WAYLAND_DISPLAY: \n {}", e);
                String::from("unknown")
            }
        }
    )
}

pub enum IpcAction {
    LockRotation(bool),
    ToggleLockRotation(),
    ChangeMonitor(String),
    ChangeTransform(TransformMapping),
    Ping(),
    Stop(),
    PrintState(),
}

pub struct Socket {
    socket: UnixListener,
    path: String,
}

pub struct Client {
    connection: UnixStream,
}

#[derive(Serialize, Deserialize)]
struct Response<T> {
    status: String,
    response: T,
}

pub type NiriSocket = niri_ipc::socket::Socket;

pub type IioNiriSocket = Socket;

impl IpcAction {
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
                    Err(e) => return Err(anyhow!("The `arg` for an action of type `change_transform` couldn't be serialized into a TransformMapping: {}", e))
                };
                Ok(IpcAction::ChangeTransform(mapping))
            }
            "ping" => Ok(IpcAction::Ping()),
            "stop" => Ok(IpcAction::Stop()),
            "print_state" => Ok(IpcAction::PrintState()),
            _ => Err(anyhow!("The action `{}` does not exist.", action)),
        }
    }

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
    pub fn new(status: String, value: T) -> Self {
        Self {
            status,
            response: value,
        }
    }
}

impl Socket {
    pub fn bind(socket_path: Option<String>) -> Result<Self> {
        let path = match socket_path {
            Some(path) => path,
            None => get_iio_niri_socket_path(),
        };

        match UnixListener::bind(path.clone()) {
            Ok(s) => Ok(Self { socket: s, path }),
            Err(e) => Err(anyhow!("Couldn't bind socket at {}: \n {}", path, e)),
        }
    }

    fn handle_client(
        stream: &mut UnixStream,
        state: Arc<Mutex<State>>,
        should_stop: Arc<AtomicBool>,
    ) -> Result<()> {
        let mut buffer = String::new();
        debug!("Reading request from client...");
        if let Err(e) = stream.read_to_string(&mut buffer) {
            return Err(anyhow!(
                "Couldn't read message from incoming stream: \n {}",
                e
            ));
        };
        debug!("Request read: {}", buffer);

        let mut state = match state.lock() {
            Ok(s) => s,
            Err(_) => {
                return Err(anyhow!(
                    "Couldn't lock on state because the value is poisonned."
                ))
            }
        };

        debug!("Parsing request as JSON...");
        let json = match serde_json::from_str::<serde_json::Value>(buffer.as_str()) {
            Ok(v) => v,
            Err(e) => return Err(anyhow!("Couldn't parse the request as valid JSON: {}", e)),
        };
        debug!("Parsing succeeded.");

        debug!("Constructing response to the client");
        let response = match IpcAction::from_json(json) {
            Ok(a) => Self::run_ipc_action(&mut state, a, Arc::clone(&should_stop))?,
            Err(e) => {
                match serde_json::to_string(&Response::new(String::from("error"), e.to_string())) {
                    Ok(r) => r,
                    Err(e) => return Err(anyhow!("Couldn't parse the response to send: {}", e)),
                }
            }
        };
        debug!("Response constructed: {}", response);

        let res = match stream.write_all(response.as_bytes()) {
            Ok(()) => Ok(()),
            Err(e) => Err(anyhow!("Couldn't write a response to the stream: {}", e)),
        };

        if let Err(e) = stream.flush() {
            return Err(anyhow!("Couldn't flush the buffer: {}", e));
        }

        res
    }

    fn run_ipc_action(
        state: &mut State,
        change: IpcAction,
        should_stop: Arc<AtomicBool>,
    ) -> Result<String> {
        match change {
            IpcAction::LockRotation(b) => {
                let old = state.lock_rotation;
                state.lock_rotation = b;
                let response = match serde_json::to_string(&Response::new(String::from("ok"), old))
                {
                    Ok(s) => s,
                    Err(e) => return Err(anyhow!("Couldn't parse response to the client: {}", e)),
                };
                Ok(response)
            }
            IpcAction::ToggleLockRotation() => {
                let old = state.lock_rotation;
                state.lock_rotation = !state.lock_rotation;
                let response = match serde_json::to_string(&Response::new(String::from("ok"), old))
                {
                    Ok(s) => s,
                    Err(e) => return Err(anyhow!("Couldn't parse response to the client: {}", e)),
                };
                Ok(response)
            }
            IpcAction::ChangeMonitor(monitor) => {
                let old = state.monitor.clone();
                state.monitor = monitor.clone();
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
                let response = match serde_json::to_string(&Response::new(String::from("ok"), old))
                {
                    Ok(s) => s,
                    Err(e) => return Err(anyhow!("Couldn't parse response to the client: {}", e)),
                };
                Ok(response)
            }
            IpcAction::Ping() => {
                let response = match serde_json::to_string(&Response::new(
                    String::from("ok"),
                    String::from("Pong!"),
                )) {
                    Ok(s) => s,
                    Err(e) => return Err(anyhow!("Couldn't parse response to the client: {}", e)),
                };
                Ok(response)
            }
            IpcAction::Stop() => {
                should_stop.store(true, Ordering::Relaxed);
                let response = match serde_json::to_string(&Response::new(
                    String::from("ok"),
                    String::from("Stopping!"),
                )) {
                    Ok(s) => s,
                    Err(e) => return Err(anyhow!("Couldn't parse response to the client: {}", e)),
                };
                Ok(response)
            }
            IpcAction::PrintState() => {
                let response =
                    match serde_json::to_string(&Response::new(String::from("ok"), state)) {
                        Ok(s) => s,
                        Err(e) => {
                            return Err(anyhow!("Couldn't parse response to the client: {}", e))
                        }
                    };
                Ok(response)
            }
        }
    }

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
                Err(err) => error!("Couldn't connect to incoming stream: \n {}", err),
            }
        }
    }

    pub fn get_path(&self) -> &str {
        self.path.as_str()
    }

    pub fn destroy_socket(&self) -> Result<()> {
        debug!("Removing socket at {}", self.path);
        match fs::remove_file(&self.path) {
            Ok(()) => Ok(()),
            Err(e) => Err(anyhow!("Couldn't not destroy socket:\n {}", e)),
        }
    }
}

impl Client {
    pub fn bind(socket_path: Option<String>) -> Result<Self> {
        let path = match socket_path {
            Some(path) => path,
            None => get_iio_niri_socket_path(),
        };
        let connection = match UnixStream::connect(&path) {
            Ok(conn) => conn,
            Err(e) => return Err(anyhow!("Couldn't connect to socket {}:\n {}", &path, e)),
        };
        Ok(Self { connection })
    }

    pub fn send(&mut self, request: IpcAction) -> Result<String> {
        debug!("Writing request to client: {}", request.to_json());
        if let Err(e) = self
            .connection
            .write_all(request.to_json().to_string().as_bytes())
        {
            return Err(anyhow!("Couldn't write message to the stream: \n {}", e));
        }
        debug!("Request sent.");

        debug!("Flushing the stream...");
        if let Err(e) = self.connection.flush() {
            return Err(anyhow!("Couldn't flush the buffer: {}", e));
        }
        debug!("Stream flushed.");

        let mut response = String::new();
        loop {
            match self.connection.read_to_string(&mut response) {
                Ok(_) => return Ok(response),
                Err(e) => {
                    if e.kind() == ErrorKind::WouldBlock {
                        continue;
                    }
                    return Err(anyhow!("{}", e));
                }
            };
        }
    }

    pub fn send_from_args(&mut self, args: MsgArgs) -> Result<()> {
        let response = (match args.command {
            MsgSubcommandArgs::LockRotation(sub_command) => {
                self.send(IpcAction::LockRotation(sub_command.lock_rotation))
            }
            MsgSubcommandArgs::ToggleLockRotation(_) => self.send(IpcAction::ToggleLockRotation()),
            MsgSubcommandArgs::Monitor(sub_command) => {
                self.send(IpcAction::ChangeMonitor(sub_command.monitor))
            }
            MsgSubcommandArgs::Transform(sub_command) => self.send(IpcAction::ChangeTransform(
                TransformMapping::from_transform_vec(Some(sub_command.transform)),
            )),
            MsgSubcommandArgs::Ping(_) => self.send(IpcAction::Ping()),
            MsgSubcommandArgs::Stop(_) => self.send(IpcAction::Stop()),
            MsgSubcommandArgs::PrintState(_) => self.send(IpcAction::PrintState()),
        })?;
        println!("{}", response);
        Ok(())
    }
}
