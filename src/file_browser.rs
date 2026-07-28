use std::env;
use std::io::Read;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::thread;

const THUNAR_BINARY: &str = "thunar";
const SCREENSHOTS_DIRECTORY_NAME: &str = "Pictures";

pub fn open_screenshots_directory() -> Result<(), String> {
    require_display_for_file_browser()?;
    let screenshots_directory = screenshots_directory_path()?;

    let child = Command::new(THUNAR_BINARY)
        .arg(&screenshots_directory)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| format!("failed to spawn {THUNAR_BINARY}: {error}"))?;

    reap_child_in_background(child, THUNAR_BINARY);
    Ok(())
}

fn screenshots_directory_path() -> Result<PathBuf, String> {
    let home_directory = env::var_os("HOME")
        .map(PathBuf::from)
        .ok_or_else(|| "HOME is not set".to_string())?;
    let screenshots_directory = home_directory.join(SCREENSHOTS_DIRECTORY_NAME);
    if !screenshots_directory.is_dir() {
        return Err(format!(
            "screenshots directory missing: {}",
            screenshots_directory.display()
        ));
    }
    Ok(screenshots_directory)
}

fn require_display_for_file_browser() -> Result<(), String> {
    if env::var_os("DISPLAY").is_some() || env::var_os("WAYLAND_DISPLAY").is_some() {
        Ok(())
    } else {
        Err("file browser needs DISPLAY or WAYLAND_DISPLAY".to_string())
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
                println!("ok: file browser finished ({label})");
            }
            Ok(status) => {
                let stderr_text = stderr_text.trim();
                if stderr_text.is_empty() {
                    eprintln!("error: file browser {label} exited with {status}");
                } else {
                    eprintln!("error: file browser {label} exited with {status}: {stderr_text}");
                }
            }
            Err(error) => {
                eprintln!("error: failed waiting for file browser {label}: {error}");
            }
        }
    });
}
