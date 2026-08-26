use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const DRM_CLASS_DIRECTORY: &str = "/sys/class/drm";
const COMMAND_TIMEOUT_SECONDS: u64 = 1;
const TIMEOUT_BINARY: &str = "timeout";

const SCREENSAVER_TARGETS: &[ScreensaverDbusTarget] = &[
    ScreensaverDbusTarget {
        destination: "org.xfce.ScreenSaver",
        object_path: "/org/xfce/ScreenSaver",
        interface: "org.xfce.ScreenSaver",
    },
    ScreensaverDbusTarget {
        destination: "org.freedesktop.ScreenSaver",
        object_path: "/org/freedesktop/ScreenSaver",
        interface: "org.freedesktop.ScreenSaver",
    },
    ScreensaverDbusTarget {
        destination: "org.freedesktop.ScreenSaver",
        object_path: "/ScreenSaver",
        interface: "org.freedesktop.ScreenSaver",
    },
    ScreensaverDbusTarget {
        destination: "org.gnome.ScreenSaver",
        object_path: "/org/gnome/ScreenSaver",
        interface: "org.gnome.ScreenSaver",
    },
    ScreensaverDbusTarget {
        destination: "org.gnome.ScreenSaver",
        object_path: "/ScreenSaver",
        interface: "org.gnome.ScreenSaver",
    },
    ScreensaverDbusTarget {
        destination: "org.kde.screensaver",
        object_path: "/ScreenSaver",
        interface: "org.freedesktop.ScreenSaver",
    },
];

#[derive(Clone, Copy)]
struct ScreensaverDbusTarget {
    destination: &'static str,
    object_path: &'static str,
    interface: &'static str,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DisplayPowerObservation {
    pub is_power_saving: bool,
    pub reason: String,
}

impl DisplayPowerObservation {
    fn active() -> Self {
        Self {
            is_power_saving: false,
            reason: "display is on".to_string(),
        }
    }

    fn power_saving(reason: impl Into<String>) -> Self {
        Self {
            is_power_saving: true,
            reason: reason.into(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct DrmConnectorState {
    connected: bool,
    enabled: bool,
    dpms_on: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct SessionLockProperties {
    locked: bool,
    is_seated_user_session: bool,
}

pub fn query_display_power_state() -> DisplayPowerObservation {
    if matches!(drm_connected_outputs_are_in_power_saving(), Some(true)) {
        return DisplayPowerObservation::power_saving(
            "connected displays report DPMS standby/suspend/off",
        );
    }

    if matches!(x11_dpms_monitor_is_in_power_saving(), Some(true)) {
        return DisplayPowerObservation::power_saving("X11 DPMS monitor is not on");
    }

    if matches!(screensaver_is_active(), Some(true)) {
        return DisplayPowerObservation::power_saving("screensaver is active");
    }

    if matches!(seated_session_is_locked(), Some(true)) {
        return DisplayPowerObservation::power_saving("graphical session is locked");
    }

    DisplayPowerObservation::active()
}

pub fn request_display_wake() {
    wake_x11_dpms();
    simulate_screensaver_user_activity();
}

fn drm_connected_outputs_are_in_power_saving() -> Option<bool> {
    connected_enabled_outputs_are_in_power_saving(&read_drm_connector_states_from(Path::new(
        DRM_CLASS_DIRECTORY,
    )))
}

fn connected_enabled_outputs_are_in_power_saving(connectors: &[DrmConnectorState]) -> Option<bool> {
    let active_outputs: Vec<&DrmConnectorState> = connectors
        .iter()
        .filter(|connector| connector.connected && connector.enabled)
        .collect();
    if active_outputs.is_empty() {
        return None;
    }
    Some(active_outputs.iter().all(|connector| !connector.dpms_on))
}

fn read_drm_connector_states_from(drm_directory: &Path) -> Vec<DrmConnectorState> {
    let Ok(entries) = fs::read_dir(drm_directory) else {
        return Vec::new();
    };

    let mut connector_states = Vec::new();
    for entry in entries.flatten() {
        let connector_directory = entry.path();
        let Some(connector_name) = connector_directory
            .file_name()
            .and_then(|name| name.to_str())
        else {
            continue;
        };
        if !is_physical_drm_connector_name(connector_name) {
            continue;
        }

        let dpms_path = connector_directory.join("dpms");
        if !dpms_path.is_file() {
            continue;
        }

        let enabled_text = read_sysfs_trimmed(connector_directory.join("enabled"));
        let status_text = read_sysfs_trimmed(connector_directory.join("status"));
        let dpms_text = read_sysfs_trimmed(dpms_path);

        connector_states.push(DrmConnectorState {
            connected: status_text
                .as_deref()
                .is_some_and(|value| value.eq_ignore_ascii_case("connected")),
            enabled: enabled_text
                .as_deref()
                .is_some_and(|value| value.eq_ignore_ascii_case("enabled")),
            dpms_on: dpms_text
                .as_deref()
                .is_some_and(|value| value.eq_ignore_ascii_case("On")),
        });
    }
    connector_states
}

fn is_physical_drm_connector_name(name: &str) -> bool {
    if !name.starts_with("card") || !name.contains('-') {
        return false;
    }
    let lowered_name = name.to_ascii_lowercase();
    !lowered_name.contains("writeback") && !lowered_name.contains("virtual")
}

fn read_sysfs_trimmed(path: PathBuf) -> Option<String> {
    fs::read_to_string(path)
        .ok()
        .map(|value| value.trim().to_string())
}

fn x11_dpms_monitor_is_in_power_saving() -> Option<bool> {
    if std::env::var_os("DISPLAY").is_none() {
        return None;
    }
    let output_text = run_command_with_timeout("xset", &["q"], COMMAND_TIMEOUT_SECONDS).ok()?;
    parse_xset_monitor_is_in_power_saving(&output_text)
}

fn parse_xset_monitor_is_in_power_saving(xset_query_text: &str) -> Option<bool> {
    for line in xset_query_text.lines() {
        let trimmed_line = line.trim();
        if trimmed_line == "Monitor is On" {
            return Some(false);
        }
        if trimmed_line == "Monitor is Off"
            || trimmed_line == "Monitor is in Standby"
            || trimmed_line == "Monitor is in Suspend"
        {
            return Some(true);
        }
    }
    None
}

fn screensaver_is_active() -> Option<bool> {
    for target in SCREENSAVER_TARGETS {
        match query_screensaver_active(target) {
            Some(is_active) => return Some(is_active),
            None => continue,
        }
    }
    None
}

fn query_screensaver_active(target: &ScreensaverDbusTarget) -> Option<bool> {
    let output_text = run_command_with_timeout(
        "busctl",
        &[
            "--user",
            "call",
            target.destination,
            target.object_path,
            target.interface,
            "GetActive",
        ],
        COMMAND_TIMEOUT_SECONDS,
    )
    .ok()?;
    parse_dbus_boolean_payload(&output_text)
}

fn parse_dbus_boolean_payload(text: &str) -> Option<bool> {
    for token in text.split_whitespace() {
        match token {
            "true" => return Some(true),
            "false" => return Some(false),
            _ => {}
        }
    }
    None
}

fn seated_session_is_locked() -> Option<bool> {
    let user_id = current_user_id().ok()?;
    let sessions_text = run_command_with_timeout(
        "loginctl",
        &["show-user", &user_id.to_string(), "--property=Sessions"],
        COMMAND_TIMEOUT_SECONDS,
    )
    .ok()?;
    let session_ids = parse_sessions_property(&sessions_text)?;

    let mut saw_unlocked_seated_session = false;
    for session_id in session_ids {
        let properties_text = run_command_with_timeout(
            "loginctl",
            &[
                "show-session",
                &session_id,
                "--property=LockedHint",
                "--property=Class",
                "--property=Seat",
            ],
            COMMAND_TIMEOUT_SECONDS,
        )
        .ok()?;
        let Some(properties) = parse_session_lock_properties(&properties_text) else {
            continue;
        };
        if !properties.is_seated_user_session {
            continue;
        }
        if properties.locked {
            return Some(true);
        }
        saw_unlocked_seated_session = true;
    }

    if saw_unlocked_seated_session {
        Some(false)
    } else {
        None
    }
}

fn parse_sessions_property(text: &str) -> Option<Vec<String>> {
    for line in text.lines() {
        let Some(session_list) = line.strip_prefix("Sessions=") else {
            continue;
        };
        let session_ids: Vec<String> = session_list
            .split_whitespace()
            .filter(|session_id| !session_id.is_empty())
            .map(str::to_string)
            .collect();
        if session_ids.is_empty() {
            return None;
        }
        return Some(session_ids);
    }
    None
}

fn parse_session_lock_properties(text: &str) -> Option<SessionLockProperties> {
    let mut locked = None;
    let mut class = None;
    let mut seat = None;

    for line in text.lines() {
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        match key {
            "LockedHint" => locked = Some(value == "yes"),
            "Class" => class = Some(value.to_string()),
            "Seat" => seat = Some(value.to_string()),
            _ => {}
        }
    }

    Some(SessionLockProperties {
        locked: locked?,
        is_seated_user_session: class.as_deref() == Some("user")
            && seat.as_deref().is_some_and(|value| !value.is_empty()),
    })
}

fn current_user_id() -> Result<u32, String> {
    let status_text = fs::read_to_string("/proc/self/status")
        .map_err(|error| format!("read /proc/self/status: {error}"))?;
    for line in status_text.lines() {
        if let Some(rest) = line.strip_prefix("Uid:") {
            let real_uid = rest.split_whitespace().next().unwrap_or_default();
            return real_uid
                .parse()
                .map_err(|error| format!("parse uid: {error}"));
        }
    }
    Err("Uid field missing from /proc/self/status".to_string())
}

fn wake_x11_dpms() {
    if std::env::var_os("DISPLAY").is_none() {
        return;
    }
    let _ = run_command_with_timeout("xset", &["dpms", "force", "on"], COMMAND_TIMEOUT_SECONDS);
}

fn simulate_screensaver_user_activity() {
    for target in SCREENSAVER_TARGETS {
        if run_command_with_timeout(
            "busctl",
            &[
                "--user",
                "call",
                target.destination,
                target.object_path,
                target.interface,
                "SimulateUserActivity",
            ],
            COMMAND_TIMEOUT_SECONDS,
        )
        .is_ok()
        {
            return;
        }
    }
}

fn run_command_with_timeout(
    program: &str,
    arguments: &[&str],
    timeout_seconds: u64,
) -> Result<String, String> {
    let timeout_seconds_text = timeout_seconds.to_string();
    let command_output = Command::new(TIMEOUT_BINARY)
        .arg("--kill-after=1s")
        .arg(&timeout_seconds_text)
        .arg(program)
        .args(arguments)
        .output()
        .map_err(|error| format!("failed to spawn {TIMEOUT_BINARY} {program}: {error}"))?;

    if command_output.status.code() == Some(124) {
        return Err(format!(
            "{program} timed out after {timeout_seconds} seconds"
        ));
    }
    if !command_output.status.success() {
        let stderr_text = String::from_utf8_lossy(&command_output.stderr);
        let stderr_text = stderr_text.trim();
        if stderr_text.is_empty() {
            return Err(format!(
                "{program} exited with status {}",
                command_output.status
            ));
        }
        return Err(format!("{program} failed: {stderr_text}"));
    }

    Ok(String::from_utf8_lossy(&command_output.stdout).into_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn drm_power_saving_when_all_connected_enabled_outputs_are_off() {
        let connectors = vec![
            DrmConnectorState {
                connected: true,
                enabled: true,
                dpms_on: false,
            },
            DrmConnectorState {
                connected: false,
                enabled: false,
                dpms_on: false,
            },
        ];
        assert_eq!(
            connected_enabled_outputs_are_in_power_saving(&connectors),
            Some(true)
        );
    }

    #[test]
    fn drm_active_when_any_connected_enabled_output_is_on() {
        let connectors = vec![
            DrmConnectorState {
                connected: true,
                enabled: true,
                dpms_on: true,
            },
            DrmConnectorState {
                connected: true,
                enabled: true,
                dpms_on: false,
            },
        ];
        assert_eq!(
            connected_enabled_outputs_are_in_power_saving(&connectors),
            Some(false)
        );
    }

    #[test]
    fn drm_unknown_when_no_connected_enabled_outputs() {
        let connectors = vec![DrmConnectorState {
            connected: false,
            enabled: false,
            dpms_on: false,
        }];
        assert_eq!(
            connected_enabled_outputs_are_in_power_saving(&connectors),
            None
        );
    }

    #[test]
    fn physical_connector_names_skip_cards_and_writeback() {
        assert!(is_physical_drm_connector_name("card0-HDMI-A-1"));
        assert!(is_physical_drm_connector_name("card1-eDP-1"));
        assert!(!is_physical_drm_connector_name("card0"));
        assert!(!is_physical_drm_connector_name("card0-Writeback-1"));
        assert!(!is_physical_drm_connector_name("renderD128"));
    }

    #[test]
    fn xset_monitor_on_is_not_power_saving() {
        let text = "DPMS (Energy Star):\n  DPMS is Enabled\n  Monitor is On\n";
        assert_eq!(parse_xset_monitor_is_in_power_saving(text), Some(false));
    }

    #[test]
    fn xset_monitor_off_is_power_saving() {
        assert_eq!(
            parse_xset_monitor_is_in_power_saving("  Monitor is Off\n"),
            Some(true)
        );
        assert_eq!(
            parse_xset_monitor_is_in_power_saving("  Monitor is in Standby\n"),
            Some(true)
        );
        assert_eq!(
            parse_xset_monitor_is_in_power_saving("  Monitor is in Suspend\n"),
            Some(true)
        );
    }

    #[test]
    fn dbus_boolean_payloads() {
        assert_eq!(parse_dbus_boolean_payload("b true\n"), Some(true));
        assert_eq!(parse_dbus_boolean_payload("b false\n"), Some(false));
        assert_eq!(parse_dbus_boolean_payload("   boolean true\n"), Some(true));
        assert_eq!(parse_dbus_boolean_payload("nope"), None);
    }

    #[test]
    fn sessions_property_splits_ids() {
        assert_eq!(
            parse_sessions_property("Sessions=36 6 7\n"),
            Some(vec!["36".to_string(), "6".to_string(), "7".to_string()])
        );
        assert_eq!(parse_sessions_property("Sessions=\n"), None);
    }

    #[test]
    fn seated_user_session_lock_properties() {
        let locked_seat =
            parse_session_lock_properties("LockedHint=yes\nClass=user\nSeat=seat0\n").unwrap();
        assert!(locked_seat.locked);
        assert!(locked_seat.is_seated_user_session);

        let user_manager =
            parse_session_lock_properties("LockedHint=no\nClass=user\nSeat=\n").unwrap();
        assert!(!user_manager.locked);
        assert!(!user_manager.is_seated_user_session);
    }
}
