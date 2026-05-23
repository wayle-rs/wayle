//! Line-delimited JSON IPC client for Triad.

use std::{env, path::PathBuf};

use serde_json::{Map, Value, json};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::UnixStream,
};

use crate::{
    error::{Error, Result, SocketKind},
    types::{RawLayoutState, RawState, RawWindow, TriadEvent},
};

pub(crate) struct TriadCommandClient {
    socket_path: PathBuf,
}

impl TriadCommandClient {
    pub(crate) fn connect() -> Result<Self> {
        Ok(Self {
            socket_path: triad_socket_path()?,
        })
    }

    pub(crate) async fn state(&self) -> Result<RawState> {
        let triad = self
            .request(json!({"triad":{"version":1,"request":"state"}}))
            .await?;
        let state = triad
            .get("state")
            .cloned()
            .ok_or(Error::UnexpectedResponse { request: "state" })?;
        serde_json::from_value(state).map_err(Error::from)
    }

    pub(crate) async fn dispatch_action(
        &self,
        action: &str,
        mut extra: Map<String, Value>,
    ) -> Result<()> {
        let mut triad = Map::new();
        triad.insert("version".into(), json!(1));
        triad.insert("request".into(), json!("action"));
        triad.insert("action".into(), json!(action));
        triad.append(&mut extra);

        self.request(Value::Object(Map::from_iter([(
            "triad".into(),
            Value::Object(triad),
        )])))
        .await?;
        Ok(())
    }

    async fn request(&self, request: Value) -> Result<Value> {
        let stream = UnixStream::connect(&self.socket_path)
            .await
            .map_err(|source| Error::IpcConnectionFailed {
                kind: SocketKind::Command,
                source,
            })?;
        let mut stream = BufReader::new(stream);
        let mut serialized = serde_json::to_string(&request)?;
        serialized.push('\n');
        stream.get_mut().write_all(serialized.as_bytes()).await?;

        let mut reply_line = String::new();
        if stream.read_line(&mut reply_line).await? == 0 {
            return Err(Error::SocketClosed {
                kind: SocketKind::Command,
            });
        }

        decode_reply(&reply_line)
    }
}

pub(crate) struct EventStream {
    reader: BufReader<UnixStream>,
}

impl EventStream {
    pub(crate) async fn connect() -> Result<Self> {
        let socket_path = triad_socket_path()?;
        let stream = UnixStream::connect(&socket_path).await.map_err(|source| {
            Error::IpcConnectionFailed {
                kind: SocketKind::EventStream,
                source,
            }
        })?;
        let mut reader = BufReader::new(stream);
        let request = json!({
            "triad": {
                "version": 1,
                "request": "event-stream",
                "events": ["layout", "state", "window"]
            }
        });
        let mut serialized = serde_json::to_string(&request)?;
        serialized.push('\n');
        reader.get_mut().write_all(serialized.as_bytes()).await?;

        let mut ack_line = String::new();
        if reader.read_line(&mut ack_line).await? == 0 {
            return Err(Error::SocketClosed {
                kind: SocketKind::EventStream,
            });
        }
        decode_reply(&ack_line)?;

        Ok(Self { reader })
    }

    pub(crate) async fn next_message(&mut self) -> Result<Option<EventMessage>> {
        let mut line = String::new();
        if self.reader.read_line(&mut line).await? == 0 {
            return Ok(None);
        }
        decode_event(&line)
    }
}

pub(crate) enum EventMessage {
    State(RawState),
    Layout(RawLayoutState),
    Window(RawWindow),
}

impl EventMessage {
    pub(crate) fn event(&self) -> TriadEvent {
        match self {
            Self::State(_) => TriadEvent::StateChanged,
            Self::Layout(_) => TriadEvent::LayoutStateChanged,
            Self::Window(window) => TriadEvent::WindowChanged {
                window_id: Some(window.id),
            },
        }
    }
}

fn decode_reply(line: &str) -> Result<Value> {
    let reply: Value = serde_json::from_str(line)?;
    if reply.get("ok").and_then(Value::as_bool) == Some(false) {
        let message = reply
            .get("error")
            .and_then(Value::as_str)
            .unwrap_or("request failed")
            .to_string();
        return Err(Error::TriadRejected(message));
    }

    reply
        .get("triad")
        .cloned()
        .ok_or(Error::UnexpectedResponse { request: "triad" })
}

fn decode_event(line: &str) -> Result<Option<EventMessage>> {
    let reply: Value = serde_json::from_str(line)?;
    let Some(triad) = reply.get("triad") else {
        return Ok(None);
    };

    match triad.get("event").and_then(Value::as_str) {
        Some("state-changed") => {
            let state = triad
                .get("state")
                .cloned()
                .ok_or(Error::UnexpectedResponse {
                    request: "state event",
                })?;
            serde_json::from_value(state)
                .map(EventMessage::State)
                .map(Some)
                .map_err(Error::from)
        }
        Some("layout-state-changed") => {
            let state = triad
                .get("state")
                .cloned()
                .ok_or(Error::UnexpectedResponse {
                    request: "layout event",
                })?;
            serde_json::from_value(state)
                .map(EventMessage::Layout)
                .map(Some)
                .map_err(Error::from)
        }
        Some("window-changed") => {
            let window = triad
                .get("window")
                .cloned()
                .ok_or(Error::UnexpectedResponse {
                    request: "window event",
                })?;
            serde_json::from_value(window)
                .map(EventMessage::Window)
                .map(Some)
                .map_err(Error::from)
        }
        _ => Ok(None),
    }
}

fn triad_socket_path() -> Result<PathBuf> {
    if let Ok(path) = env::var("TRIAD_SOCKET") {
        if !path.is_empty() {
            return Ok(PathBuf::from(path));
        }
    }

    env::var_os("XDG_RUNTIME_DIR")
        .map(PathBuf::from)
        .map(|dir| dir.join("triad.sock"))
        .or_else(|| Some(PathBuf::from("/tmp/triad.sock")))
        .ok_or(Error::TriadNotRunning)
}
