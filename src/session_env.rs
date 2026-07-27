use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

const GUI_ENVIRONMENT_KEYS: &[&str] = &[
    "DISPLAY",
    "WAYLAND_DISPLAY",
    "XAUTHORITY",
    "DBUS_SESSION_BUS_ADDRESS",
    "XDG_RUNTIME_DIR",
    "XDG_SESSION_TYPE",
];

const SESSION_PROCESS_NAMES: &[&str] = &[
    "xfce4-session",
    "gnome-session",
    "gnome-session-b",
    "plasma_session",
    "plasmashell",
    "sway",
    "Hyprland",
    "river",
    "xmonad",
    "i3",
    "openbox",
    "lxsession",
    "mate-session",
    "cinnamon-session",
];

pub fn apply_session_gui_environment() -> Result<(), String> {
    let resolved_variables = resolve_gui_environment_variables()?;
    for (key, value) in resolved_variables {
        if std::env::var_os(&key).is_none() {
            set_environment_variable(&key, &value);
            println!("session env: set {key}={value}");
        }
    }

    if std::env::var_os("DISPLAY").is_none() && std::env::var_os("WAYLAND_DISPLAY").is_none() {
        return Err(
            "no DISPLAY or WAYLAND_DISPLAY available; start from a graphical session or export DISPLAY"
                .to_string(),
        );
    }

    Ok(())
}

fn set_environment_variable(key: &str, value: &str) {
    unsafe {
        std::env::set_var(key, value);
    }
}

fn resolve_gui_environment_variables() -> Result<HashMap<String, String>, String> {
    let mut resolved = HashMap::new();

    for key in GUI_ENVIRONMENT_KEYS {
        if let Ok(value) = std::env::var(key) {
            resolved.insert((*key).to_string(), value);
        }
    }

    if resolved.contains_key("DISPLAY") || resolved.contains_key("WAYLAND_DISPLAY") {
        return Ok(resolved);
    }

    let current_user_id = read_self_uid()?;
    for process_id in list_process_ids()? {
        if process_owner_uid(process_id) != Some(current_user_id) {
            continue;
        }
        let Some(process_name) = process_comm(process_id) else {
            continue;
        };
        if !SESSION_PROCESS_NAMES
            .iter()
            .any(|name| process_name == *name)
        {
            continue;
        }

        let process_environment = read_process_environment(process_id)?;
        for key in GUI_ENVIRONMENT_KEYS {
            if resolved.contains_key(*key) {
                continue;
            }
            if let Some(value) = process_environment.get(*key) {
                resolved.insert((*key).to_string(), value.clone());
            }
        }

        if resolved.contains_key("DISPLAY") || resolved.contains_key("WAYLAND_DISPLAY") {
            println!(
                "session env: inherited GUI variables from pid {process_id} ({process_name})"
            );
            return Ok(resolved);
        }
    }

    Ok(resolved)
}

fn read_self_uid() -> Result<u32, String> {
    process_owner_uid(read_self_pid()?)
        .ok_or_else(|| "could not determine current uid".to_string())
}

fn read_self_pid() -> Result<u32, String> {
    let status_text = fs::read_to_string("/proc/self/status")
        .map_err(|error| format!("read /proc/self/status: {error}"))?;
    for line in status_text.lines() {
        if let Some(rest) = line.strip_prefix("Pid:") {
            let pid_text = rest.split_whitespace().next().unwrap_or_default();
            return pid_text
                .parse()
                .map_err(|error| format!("parse self pid: {error}"));
        }
    }
    Err("Pid field missing from /proc/self/status".to_string())
}

fn list_process_ids() -> Result<Vec<u32>, String> {
    let mut process_ids = Vec::new();
    for entry in fs::read_dir("/proc").map_err(|error| format!("read /proc: {error}"))? {
        let entry = entry.map_err(|error| format!("read /proc entry: {error}"))?;
        if let Ok(process_id) = entry.file_name().to_string_lossy().parse::<u32>() {
            process_ids.push(process_id);
        }
    }
    Ok(process_ids)
}

fn process_owner_uid(process_id: u32) -> Option<u32> {
    let status_path = PathBuf::from(format!("/proc/{process_id}/status"));
    let status_text = fs::read_to_string(status_path).ok()?;
    for line in status_text.lines() {
        if let Some(rest) = line.strip_prefix("Uid:") {
            let real_uid = rest.split_whitespace().next()?;
            return real_uid.parse().ok();
        }
    }
    None
}

fn process_comm(process_id: u32) -> Option<String> {
    let comm_path = PathBuf::from(format!("/proc/{process_id}/comm"));
    let comm_text = fs::read_to_string(comm_path).ok()?;
    Some(comm_text.trim().to_string())
}

fn read_process_environment(process_id: u32) -> Result<HashMap<String, String>, String> {
    let environ_path = PathBuf::from(format!("/proc/{process_id}/environ"));
    let environ_bytes = fs::read(&environ_path)
        .map_err(|error| format!("read {}: {error}", environ_path.display()))?;

    let mut environment = HashMap::new();
    for entry in environ_bytes.split(|byte| *byte == 0) {
        if entry.is_empty() {
            continue;
        }
        let Ok(entry_text) = std::str::from_utf8(entry) else {
            continue;
        };
        let Some((key, value)) = entry_text.split_once('=') else {
            continue;
        };
        environment.insert(key.to_string(), value.to_string());
    }
    Ok(environment)
}
