#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::env;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use serde_json::json;

use crate::chrome_debugger::ChromeRemoteDebugger;

mod chrome_debugger;

fn locate_lunar_executable() -> Result<String> {
    let exe = match env::consts::OS {
        "windows" => env::var("localappdata")? + r"\Programs\lunarclient\Lunar Client.exe",
        "macos" => "/Applications/Lunar Client.app/Contents/MacOS/Lunar Client".into(),
        "linux" => "/usr/bin/lunarclient".into(),
        os => bail!("automatically locating lunar is not supported on {os}")
    };

    if !Path::new(&exe).exists() {
        bail!("'{}' does not exist", exe);
    }

    Ok(exe)
}

fn main() -> Result<()> {
    let lunar_exe = match env::args().nth(1) {
        Some(path) => path,
        None => locate_lunar_executable().context("failed to locate the lunar executable, try passing it by argument")?
    };

    let current_exe = env::current_exe()?;
    let dir = dunce::simplified(
        current_exe.parent().context("executable has no parent directory")?
    ).to_str().context("executable path contains invalid utf8")?;

    let mut debugger = ChromeRemoteDebugger::spawn_process_and_connect(&lunar_exe)?;
    debugger.send("Runtime.callFunctionOn", json!({
        "functionDeclaration": include_str!("inject.js"),
        "arguments": [
            {
                "value": dir
            }
        ],
        "executionContextId": 1
    }))?;

    Ok(())
}