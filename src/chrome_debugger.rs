use std::error::Error;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{Ipv4Addr, TcpStream};
use serde::Deserialize;
use serde_json::json;
use tungstenite::{Message, WebSocket};

pub struct ChromeDebugger {
    ws: WebSocket<TcpStream>
}

impl ChromeDebugger {
    pub fn connect(port: u16) -> Result<ChromeDebugger, Box<dyn Error>> {
        let mut stream = TcpStream::connect((Ipv4Addr::LOCALHOST, port))?;
        let ws_url = get_websocket_url(&mut stream)?;

        Ok(Self {
            ws: tungstenite::client(ws_url, stream)?.0
        })
    }

    pub fn send(&mut self, method: &str, params: serde_json::Value) -> Result<(), Box<dyn Error>> {
        self.ws.write_message(Message::Text(
            serde_json::to_string(&json!({
                "id": 1,
                "method": method,
                "params": params
            }))?
        ))?;

        if cfg!(debug_assertions) {
            println!("{}", self.ws.read_message()?);
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