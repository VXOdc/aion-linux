//! `aiond` — the AION core daemon.
//!
//! Milestone 3 scope: periodically gather a hardware report and write it
//! to /run/aion/hardware.json, where the CLI, Model Center, and other
//! consumers read it. Later milestones add the D-Bus/JSON API surface,
//! runtime supervision, and update orchestration.
//!
//! The daemon runs independently of any graphical session.

use std::path::Path;
use std::time::Duration;

const REPORT_PATH: &str = "/run/aion/hardware.json";

fn main() -> anyhow::Result<()> {
    // Refuse to do anything surprising when /run is not writable
    // (containers, early boot): retry quietly instead of crashing.
    loop {
        match write_report() {
            Ok(()) => log("hardware report refreshed"),
            Err(e) => log(&format!("report refresh failed: {e}")),
        }
        std::thread::sleep(Duration::from_secs(30));
    }
}

fn write_report() -> anyhow::Result<()> {
    let report = aion_hardware::detect_all();
    let json = serde_json::to_string_pretty(&report)?;
    let path = Path::new(REPORT_PATH);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, json)?;
    Ok(())
}

fn log(message: &str) {
    eprintln!("aiond: {message}");
}
