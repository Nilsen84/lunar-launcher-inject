use crate::chrome_debugger::CdpError;
use std::{io, net::TcpStream};

#[derive(Debug, thiserror::Error)]
pub enum DebuggerError {
    #[error("invalid inspector URL: {0}")]
    Url(#[from] tungstenite::http::uri::InvalidUri),
    #[error("inspector I/O failed: {0}")]
    Io(#[from] std::io::Error),
    #[error("inspector WebSocket failed: {0}")]
    WebSocket(#[source] Box<tungstenite::Error>),
    #[error("inspector WebSocket handshake failed: {0}")]
    Handshake(
        #[source]
        Box<
            tungstenite::HandshakeError<tungstenite::handshake::client::ClientHandshake<TcpStream>>,
        >,
    ),
    #[error("invalid inspector JSON: {0}")]
    Json(#[from] serde_json::Error),
    #[error("inspector disconnected")]
    Disconnected,
}

impl From<tungstenite::Error> for DebuggerError {
    fn from(error: tungstenite::Error) -> Self {
        Self::WebSocket(Box::new(error))
    }
}

#[derive(Debug, thiserror::Error)]
pub enum LocateError {
    #[cfg_attr(windows, error("%{0}% not defined"))]
    #[cfg_attr(not(windows), error("${0} not defined"))]
    MissingEnv(&'static str),
    #[error("unsupported os: {0}")]
    UnsupportedOs(&'static str),
    #[error("searched in the following locations:{0}")]
    NotFound(String),
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("failed to locate lunars launcher: {0}\nMake sure you have lunar installed before running lunar-launcher-inject")]
    Locate(#[from] LocateError),
    #[error("{operation}: {source}")]
    Io {
        operation: &'static str,
        #[source]
        source: io::Error,
    },
    #[error("failed to connect debugger: {0}")]
    Connect(#[source] DebuggerError),
    #[error(transparent)]
    Debugger(#[from] DebuggerError),
    #[error("failed to serialize injection payload: {0}")]
    Payload(#[from] serde_json::Error),
    #[error("CDP request {id} failed: {source}")]
    Cdp {
        id: u32,
        #[source]
        source: CdpError,
    },
    #[error("Debugger.paused did not include a call frame")]
    MissingCallFrame,
    #[error("injection threw a JavaScript exception")]
    JavaScript,
    #[error("Lunar did not publish a debugger URL within 15 seconds")]
    StartupTimeout,
    #[error("stderr reader stopped before reporting a debugger URL")]
    StderrDisconnected,
    #[error("stderr reader thread panicked")]
    StderrThreadPanicked,
    #[error("child process has no stderr pipe")]
    MissingStderr,
    #[error("injector executable has no parent directory")]
    MissingParent,
}

pub fn io_error(operation: &'static str) -> impl FnOnce(io::Error) -> Error {
    move |source| Error::Io { operation, source }
}
