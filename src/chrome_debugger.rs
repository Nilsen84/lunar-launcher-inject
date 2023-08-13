use std::error::Error;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{Ipv4Addr, TcpStream};
use std::str::FromStr;
use serde::Deserialize;
use serde_json::json;
use tungstenite::{Message, WebSocket};
use tungstenite::error::UrlError;
use tungstenite::http::Uri;

pub struct ChromeDebugger {
    id: u32,
    ws: WebSocket<TcpStream>
}

impl ChromeDebugger {
    pub fn connect_port(port: u16) -> Result<ChromeDebugger, Box<dyn Error>> {
        let mut stream = TcpStream::connect((Ipv4Addr::LOCALHOST, port))?;
        let ws_url = get_websocket_url(&mut stream)?;

        Ok(Self {
            id: 1,
            ws: tungstenite::client(ws_url, stream)?.0
        })
    }

    pub fn connect_url(uri: impl AsRef<str>) -> Result<ChromeDebugger, Box<dyn Error>> {
        let url = Uri::from_str(uri.as_ref())?;
        let host = url.host().ok_or(tungstenite::Error::Url(UrlError::NoHostName))?;
        let stream = TcpStream::connect((host, url.port_u16().unwrap_or(80)))?;

        Ok(Self {
            id: 1,
            ws: tungstenite::client(&url, stream)?.0
        })
    }

    pub fn send(&mut self, method: &str, params: serde_json::Value) -> Result<(), Box<dyn Error>> {
        self.ws.send(Message::Text(
            serde_json::to_string(&json!({
                "id": self.id,
                "method": method,
                "params": params
            }))?
        ))?;

        self.id += 1;

        if cfg!(debug_assertions) {
            println!("{}", self.ws.read()?);
        }

        Ok(())
    }
}

fn get_websocket_url(stream: &mut TcpStream) -> Result<String, Box<dyn Error>> {
    stream.write_all(b"GET /json/list HTTP/1.1\r\nHost: localhost\r\n\r\n")?;

    let mut reader = BufReader::new(stream);
    for line in reader.by_ref().lines() {
        if line?.is_empty() { break }
    }

    #[derive(Deserialize)]
    struct Target {
        #[serde(rename = "webSocketDebuggerUrl")]
        ws_url: String
    }

    let mut de = serde_json::Deserializer::from_reader(reader);
    let mut targets: Vec<Target> = Vec::deserialize(&mut de)?;
    Ok(targets.pop().ok_or("no debugging targets found")?.ws_url)
}