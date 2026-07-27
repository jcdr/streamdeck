use std::process::{Command, Stdio};

const SHELL_BINARY: &str = "sh";
const XFCE4_SCREENSHOOTER_BINARY: &str = "xfce4-screenshooter";
const WINDOW_SCREENSHOT_SCRIPT: &str = "/usr/local/bin/screenshoot-window";
const REGION_SCREENSHOT_SCRIPT: &str = "/usr/local/bin/screenshoot-region";

fn spawn_detached(binary_path: &str, arguments: &[&str]) -> Result<(), String> {
    Command::new(binary_path)
        .args(arguments)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::inherit())
        .spawn()
        .map_err(|error| format!("failed to spawn {binary_path}: {error}"))?;
    Ok(())
}

fn spawn_shell_script(script_path: &str) -> Result<(), String> {
    Command::new(SHELL_BINARY)
        .arg(script_path)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::inherit())
        .spawn()
        .map_err(|error| format!("failed to spawn {SHELL_BINARY} {script_path}: {error}"))?;
    Ok(())
}

pub fn capture_full_screenshot() -> Result<(), String> {
    spawn_detached(XFCE4_SCREENSHOOTER_BINARY, &[])
}

pub fn capture_active_window_screenshot() -> Result<(), String> {
    spawn_shell_script(WINDOW_SCREENSHOT_SCRIPT)
}

pub fn capture_region_screenshot() -> Result<(), String> {
    spawn_shell_script(REGION_SCREENSHOT_SCRIPT)
}
