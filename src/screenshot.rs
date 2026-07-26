use std::process::{Command, Stdio};

const XFCE4_SCREENSHOOTER_BINARY: &str = "xfce4-screenshooter";
const WINDOW_SCREENSHOT_SCRIPT: &str = "/usr/local/bin/screenshoot-window";
const REGION_SCREENSHOT_SCRIPT: &str = "/usr/local/bin/screenshoot-region";

fn spawn_detached(binary_path: &str, arguments: &[&str]) -> Result<(), String> {
    Command::new(binary_path)
        .args(arguments)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|error| format!("failed to spawn {binary_path}: {error}"))?;
    Ok(())
}

pub fn capture_full_screenshot() -> Result<(), String> {
    spawn_detached(XFCE4_SCREENSHOOTER_BINARY, &[])
}

pub fn capture_active_window_screenshot() -> Result<(), String> {
    spawn_detached(WINDOW_SCREENSHOT_SCRIPT, &[])
}

pub fn capture_region_screenshot() -> Result<(), String> {
    spawn_detached(REGION_SCREENSHOT_SCRIPT, &[])
}
