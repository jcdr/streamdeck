use std::io::Read;
use std::process::{Child, Command, Stdio};
use std::thread;

const SHELL_BINARY: &str = "sh";
const XFCE4_SCREENSHOOTER_BINARY: &str = "xfce4-screenshooter";
const WINDOW_SCREENSHOT_SCRIPT: &str = "/usr/local/bin/screenshoot-window";
const REGION_SCREENSHOT_SCRIPT: &str = "/usr/local/bin/screenshoot-region";

fn spawn_detached(binary_path: &str, arguments: &[&str]) -> Result<(), String> {
    require_display_for_screenshots()?;

    let child = Command::new(binary_path)
        .args(arguments)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| format!("failed to spawn {binary_path}: {error}"))?;

    reap_child_in_background(child, binary_path);
    Ok(())
}

fn spawn_shell_script(script_path: &str) -> Result<(), String> {
    require_display_for_screenshots()?;

    let child = Command::new(SHELL_BINARY)
        .arg(script_path)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| format!("failed to spawn {SHELL_BINARY} {script_path}: {error}"))?;

    reap_child_in_background(child, script_path);
    Ok(())
}

fn require_display_for_screenshots() -> Result<(), String> {
    if std::env::var_os("DISPLAY").is_some() || std::env::var_os("WAYLAND_DISPLAY").is_some() {
        Ok(())
    } else {
        Err("screenshot needs DISPLAY or WAYLAND_DISPLAY".to_string())
    }
}

fn reap_child_in_background(mut child: Child, label: &str) {
    let label = label.to_string();
    thread::spawn(move || {
        let mut stderr_text = String::new();
        if let Some(mut stderr) = child.stderr.take() {
            let _ = stderr.read_to_string(&mut stderr_text);
        }

        match child.wait() {
            Ok(status) if status.success() => {
                println!("ok: screenshot process finished ({label})");
            }
            Ok(status) => {
                let stderr_text = stderr_text.trim();
                if stderr_text.is_empty() {
                    eprintln!("error: screenshot process {label} exited with {status}");
                } else {
                    eprintln!(
                        "error: screenshot process {label} exited with {status}: {stderr_text}"
                    );
                }
            }
            Err(error) => {
                eprintln!("error: failed waiting for screenshot process {label}: {error}");
            }
        }
    });
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
