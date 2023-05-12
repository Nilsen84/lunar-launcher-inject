#![feature(try_blocks)]

use std::{env, io};
use std::error::Error;
use std::io::{BufRead, BufReader, ErrorKind};
use std::net::{Ipv4Addr, TcpListener};
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::string::String;
use std::thread::sleep;
use std::time::Duration;

use serde_json::json;

use crate::chrome_debugger::ChromeDebugger;

mod chrome_debugger;

fn free_port() -> io::Result<u16> {
    Ok(TcpListener::bind((Ipv4Addr::LOCALHOST, 0))?.local_addr()?.port())
}

fn find_lunar_executable() -> Result<String, String> {
    let exe = match env::consts::OS {
        "windows" => env::var("localappdata").or(Err("%localappdata% not defined"))?
            + r"\Programs\lunarclient\Lunar Client.exe",
        "macos" => "/Applications/Lunar Client.app/Contents/MacOS/Lunar Client".into(),
        "linux" => "/usr/bin/lunarclient".into(),
        _ => Err("unsupported os")?
    };

    if !Path::new(&exe).exists() {
        Err(format!("'{}' does not exist", exe))?
    }

    Ok(exe)
}

fn wait_for_devtools_server(cmd: &mut Child) -> io::Result<()> {
    let reader = BufReader::new(cmd.stderr.take().unwrap());
    for line in reader.lines() {
        if line?.starts_with("DevTools listening on ") {
            return Ok(())
        }
    }

    Err(io::Error::new(ErrorKind::UnexpectedEof, "'DevTools listening on ' was never printed"))
}

fn run() -> Result<(), Box<dyn Error>> {
    let lunar_exe = match env::args().nth(1) {
        Some(arg) => arg,
        _ => find_lunar_executable().map_err(|e|
            format!("failed to locate lunars launcher, try passing the path to its executable by argument: {}", e)
        )?
    };

    let port = free_port()?;

    let mut cp = Command::new(lunar_exe)
        .arg(format!("--remote-debugging-port={}", port))
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("failed to start lunar: {}", e))?;

    let res = try {
        wait_for_devtools_server(&mut cp)?;
        // on windows the launcher gets stuck on a black screen if you inject code too early
        // no idea why
        sleep(Duration::from_millis(1000));

        let mut debugger = ChromeDebugger::connect(port).map_err(|e| format!("failed to connect debugger: {}", e))?;

        let payload = format!(
            "{}({})",
            include_str!("payload.js"),
            serde_json::to_string(env::current_exe()?.parent().unwrap())?
        );

        debugger.send("Runtime.evaluate", json!({
            "expression": payload
        }))?;
    };

    if let Err(_) = res {
        let _ = cp.kill();
    }

    res
}

fn main() {
    if let Err(e) = run() {
        eprintln!("[error] {}", e);
        std::process::exit(1);
    }
}