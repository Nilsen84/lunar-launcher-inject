use crate::error::DebuggerError as Error;
use serde::Deserialize;
use serde_json::json;
use std::net::TcpStream;
use std::str::FromStr;
use std::time::Duration;
use tungstenite::error::UrlError;
use tungstenite::http::Uri;
use tungstenite::{Message as WsMessage, WebSocket};

#[derive(Debug, Deserialize, thiserror::Error)]
#[error("{message} ({code})")]
pub struct CdpError {
    pub code: i64,
    pub message: String,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum Message {
    Success {
        id: u32,
        result: serde_json::Value,
    },
    Failure {
        id: u32,
        error: CdpError,
    },
    Event {
        method: String,
        #[serde(default)]
        params: serde_json::Value,
    },
}

pub struct ChromeDebugger {
    ws: WebSocket<TcpStream>,
}

impl ChromeDebugger {
    pub fn connect_url(uri: impl AsRef<str>) -> Result<ChromeDebugger, Error> {
        let url = Uri::from_str(uri.as_ref())?;
        let host = url
            .host()
            .ok_or(tungstenite::Error::Url(UrlError::NoHostName))?;
        let stream = TcpStream::connect((host, url.port_u16().unwrap_or(80)))?;
        stream.set_read_timeout(Some(Duration::from_secs(15)))?;
        stream.set_write_timeout(Some(Duration::from_secs(15)))?;

        Ok(Self {
            ws: tungstenite::client(&url, stream)
                .map_err(|error| Error::Handshake(Box::new(error)))?
                .0,
        })
    }

    pub fn send(&mut self, id: u32, method: &str, params: serde_json::Value) -> Result<(), Error> {
        self.ws.send(WsMessage::Text(
            serde_json::to_string(&json!({
                "id": id,
                "method": method,
                "params": params
            }))?
            .into(),
        ))?;

        Ok(())
    }

    pub fn read(&mut self) -> Result<Message, Error> {
        loop {
            match self.ws.read()? {
                WsMessage::Text(text) => {
                    return Ok(serde_json::from_str(&text)?);
                }
                WsMessage::Close(_) => return Err(Error::Disconnected),
                WsMessage::Ping(_) => self.ws.flush()?,
                _ => {}
            }
        }
    }
}
