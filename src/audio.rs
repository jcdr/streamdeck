use std::process::Command;

const WPCTL_BINARY: &str = "wpctl";
const DEFAULT_AUDIO_SINK: &str = "@DEFAULT_AUDIO_SINK@";
const MUTE_ENABLE_ARGUMENT: &str = "1";
const MUTE_DISABLE_ARGUMENT: &str = "0";
const VOLUME_STEP_UP: &str = "5%+";
const VOLUME_STEP_DOWN: &str = "5%-";

fn run_wpctl(arguments: &[&str]) -> Result<(), String> {
    let command_status = Command::new(WPCTL_BINARY)
        .args(arguments)
        .status()
        .map_err(|error| format!("failed to spawn {WPCTL_BINARY}: {error}"))?;

    if command_status.success() {
        Ok(())
    } else {
        Err(format!(
            "{WPCTL_BINARY} exited with status {command_status}"
        ))
    }
}

pub fn mute_default_sink() -> Result<(), String> {
    run_wpctl(&["set-mute", DEFAULT_AUDIO_SINK, MUTE_ENABLE_ARGUMENT])
}

pub fn unmute_default_sink() -> Result<(), String> {
    run_wpctl(&["set-mute", DEFAULT_AUDIO_SINK, MUTE_DISABLE_ARGUMENT])
}

pub fn increase_default_sink_volume() -> Result<(), String> {
    run_wpctl(&["set-volume", DEFAULT_AUDIO_SINK, VOLUME_STEP_UP])
}

pub fn decrease_default_sink_volume() -> Result<(), String> {
    run_wpctl(&["set-volume", DEFAULT_AUDIO_SINK, VOLUME_STEP_DOWN])
}

pub fn set_default_sink_volume_percent(volume_percent: u8) -> Result<(), String> {
    let volume_argument = format!("{volume_percent}%");
    run_wpctl(&["set-volume", DEFAULT_AUDIO_SINK, &volume_argument])
}
