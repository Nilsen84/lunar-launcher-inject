use std::net::{Ipv4Addr, TcpListener, TcpStream};
use std::process::Command;
use tungstenite::stream::MaybeTlsStream;
use tungstenite::{Message, WebSocket};
use serde::Deserialize;
use anyhow::{Context, Result};
use serde_json::json;

fn free_port() -> Result<u16> {
    Ok(
        TcpListener::bind((Ipv4Addr::UNSPECIFIED, 0))?.local_addr()?.port()
    )
}

pub struct ChromeRemoteDebugger {
    ws: WebSocket<MaybeTlsStream<TcpStream>>
}

impl ChromeRemoteDebugger {
    pub fn spawn_process_and_connect(prog: &str) -> Result<Self> {
        let port = free_port().context("failed to find a free tcp port")?;
        Command::new(prog)
            .arg(format!("--remote-debugging-port={port}"))
            .spawn()?;

        Self::connect(port)
    }

    pub fn connect(port: u16) -> Result<Self> {
        #[derive(Deserialize)]
        struct Target {
            #[serde(rename = "webSocketDebuggerUrl")]
            ws_url: String
        }

        let targets: Vec<Target> = reqwest::blocking::get(
            format!("http://localhost:{port}/json/list")
        )?.json()?;

        let selected = targets.first().context("no debugging targets found")?;

        let (ws, _) = tungstenite::connect(
            &selected.ws_url
        )?;

        Ok(Self{
            ws
        })
    }

    pub fn send(&mut self, method: &str, params: serde_json::Value) -> Result<()> {
        self.ws.write_message(
            Message::Text(
                json!({
                    "id": 1,
                    "method": method,
                    "params": params
                }).to_string()
            )
        )?;

        Ok(())
    }
}