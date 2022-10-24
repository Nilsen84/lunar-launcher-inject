use std::thread;
use std::net::{Ipv4Addr, TcpListener, TcpStream};
use std::process::Command;
use std::time::Duration;

use anyhow::{bail, Context, Result};
use serde::Deserialize;
use serde_json::json;
use tungstenite::{Message, WebSocket};
use tungstenite::stream::MaybeTlsStream;

fn free_port() -> Result<u16> {
    Ok(
        TcpListener::bind((Ipv4Addr::UNSPECIFIED, 0))?.local_addr()?.port()
    )
}

pub struct ChromeRemoteDebugger {
    ws: WebSocket<MaybeTlsStream<TcpStream>>
}

impl ChromeRemoteDebugger {
    pub fn spawn_process_and_connect(program: &str) -> Result<Self> {
        let port = free_port().context("failed to find a free tcp port")?;
        Command::new(program)
            .arg(format!("--remote-debugging-port={}", port))
            .spawn()?;

        Self::connect(port)
    }

    pub fn connect(port: u16) -> Result<Self> {
        #[derive(Deserialize)]
        struct Target {
            #[serde(rename = "webSocketDebuggerUrl")]
            ws_url: String
        }

        let mut targets = Vec::<Target>::new();
        let mut retry = 0;
        while targets.is_empty() {
            if retry > 3 {
                bail!("Failed to get targets");
            }
            match reqwest::blocking::get(format!("http://localhost:{}/json/list", port)) {
                Ok(resp) => {
                    targets = resp.json()?;
                }
                Err(_) => {
                    retry += 1;
                    thread::sleep(Duration::from_secs(1));
                }
            }
        }

        let selected = &targets[0];

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