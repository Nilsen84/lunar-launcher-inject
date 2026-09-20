use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::string::String;
use std::sync::mpsc;
use std::time::Duration;
use std::{env, io, thread};

use serde_json::json;

use crate::chrome_debugger::{ChromeDebugger, Message};
use crate::error::{io_error, Error, LocateError};

mod chrome_debugger;
mod error;

fn find_lunar_executable() -> Result<PathBuf, LocateError> {
    let paths: Vec<PathBuf> = match env::consts::OS {
        "windows" => {
            let localappdata = PathBuf::from(
                env::var_os("LOCALAPPDATA").ok_or(LocateError::MissingEnv("LOCALAPPDATA"))?,
            );

            vec![
                localappdata.join(r"Programs\launcher\Lunar Client.exe"),
                localappdata.join(r"Programs\lunarclient\Lunar Client.exe"),
            ]
        }
        "macos" => {
            let mut paths = vec![PathBuf::from(
                "/Applications/Lunar Client.app/Contents/MacOS/Lunar Client",
            )];
            if let Some(home) = env::var_os("HOME") {
                paths.push(
                    PathBuf::from(home)
                        .join("Applications/Lunar Client.app/Contents/MacOS/Lunar Client"),
                );
            }
            paths
        }
        "linux" => vec!["/usr/bin/lunarclient".into()],
        os => return Err(LocateError::UnsupportedOs(os)),
    };

    paths.iter().find(|p| p.exists()).cloned().ok_or_else(|| {
        let locations = paths
            .iter()
            .map(|path| format!("\n - {}", path.display()))
            .collect();
        LocateError::NotFound(locations)
    })
}

fn wait_for_websocket_url(reader: &mut impl BufRead) -> io::Result<Option<String>> {
    for line in reader.lines() {
        if let Some(url) = line?.strip_prefix("Debugger listening on ") {
            return Ok(Some(url.into()));
        }
    }
    Ok(None)
}

fn inject_url(url: &str) -> Result<(), Error> {
    println!("[LLI] Connecting to {}", url);
    let mut debugger = ChromeDebugger::connect_url(url).map_err(Error::Connect)?;

    let executable =
        env::current_exe().map_err(io_error("failed to locate injector executable"))?;

    let payload = format!(
        "{}({})",
        include_str!("payload.js"),
        serde_json::to_string(executable.parent().ok_or(Error::MissingParent)?)?
    );

    const ENABLE: u32 = 1;
    const RUN: u32 = 2;
    const EVALUATE: u32 = 3;
    const DISABLE: u32 = 4;

    debugger.send(ENABLE, "Debugger.enable", json!({}))?;
    let mut evaluating = false;
    loop {
        let message = debugger.read()?;
        println!("[CDP] {message:?}");
        match message {
            Message::Failure { id, error } => {
                return Err(Error::Cdp { id, source: error });
            }
            Message::Success { id: ENABLE, .. } => {
                debugger.send(RUN, "Runtime.runIfWaitingForDebugger", json!({}))?;
            }
            Message::Event { method, params } if method == "Debugger.paused" && !evaluating => {
                let frame = params
                    .pointer("/callFrames/0/callFrameId")
                    .and_then(serde_json::Value::as_str)
                    .ok_or(Error::MissingCallFrame)?;
                debugger.send(
                    EVALUATE,
                    "Debugger.evaluateOnCallFrame",
                    json!({
                        "callFrameId": frame,
                        "expression": payload
                    }),
                )?;
                evaluating = true;
            }
            Message::Success {
                id: EVALUATE,
                result,
            } => {
                if let Some(_) = result.get("exceptionDetails") {
                    return Err(Error::JavaScript);
                }
                // Disabling the debugger also resumes execution.
                debugger.send(DISABLE, "Debugger.disable", json!({}))?;
            }
            Message::Success { id: DISABLE, .. } => break,
            _ => {}
        }
    }

    Ok(())
}

fn run() -> Result<(), Error> {
    let lunar_exe = match env::args_os().nth(1) {
        Some(arg) => PathBuf::from(arg),
        _ => find_lunar_executable()?,
    };

    let mut cp = scopeguard::guard(
        Command::new(&lunar_exe)
            .arg("--inspect-brk=0")
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(io_error("failed to start lunar"))?,
        |mut cp| {
            let _ = cp.kill();
            let _ = cp.wait();
        },
    );

    let mut stderr = BufReader::new(cp.stderr.take().ok_or(Error::MissingStderr)?);
    let (url_tx, url_rx) = mpsc::channel();

    let stderr_thread = thread::Builder::new()
        .name("lunar-stderr".into())
        .spawn(move || {
            if let Some(result) = wait_for_websocket_url(&mut stderr).transpose() {
                let _ = url_tx.send(result);
            }
            drop(url_tx);
            io::copy(&mut stderr, &mut io::sink())
        })
        .map_err(io_error("failed to start stderr reader"))?;

    let url = match url_rx.recv_timeout(Duration::from_secs(15)) {
        Ok(Ok(url)) => url,
        Ok(Err(source)) => {
            return Err(Error::Io {
                operation: "failed to read lunar stderr",
                source,
            })
        }
        Err(mpsc::RecvTimeoutError::Timeout) => {
            return Err(Error::StartupTimeout);
        }
        Err(mpsc::RecvTimeoutError::Disconnected) => {
            return Err(Error::StderrDisconnected);
        }
    };

    inject_url(&url)?;

    let mut child = scopeguard::ScopeGuard::into_inner(cp);
    child.wait().map_err(io_error("failed to wait for lunar"))?;
    // Do not join the command thread: it may be blocked on terminal input.
    stderr_thread
        .join()
        .map_err(|_| Error::StderrThreadPanicked)?
        .map_err(io_error("failed to drain lunar stderr"))?;
    Ok(())
}

fn main() {
    if let Err(e) = run() {
        eprintln!("[error] {}", e);
        std::process::exit(1);
    }
}
