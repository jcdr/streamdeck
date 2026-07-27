use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

use elgato_streamdeck::{
    list_devices, new_hidapi, DeviceStateUpdate, StreamDeck, StreamDeckError,
};
use image::DynamicImage;

use crate::actions::DeckAction;
use crate::keys::{action_for_key_index, key_bindings};

const KEY_IMAGE_DIRECTORY_NAME: &str = "assets/keys";
const DEFAULT_BRIGHTNESS_PERCENT: u8 = 70;
const BUTTON_POLL_TIMEOUT: Duration = Duration::from_millis(100);
const DEVICE_RECONNECT_POLL_INTERVAL: Duration = Duration::from_secs(2);
const DEVICE_OPEN_RETRY_LOG_INTERVAL: Duration = Duration::from_secs(30);

pub enum ConnectedSessionOutcome {
    ShutdownRequested,
    DeviceDisconnected,
}

pub struct DeckRuntime {
    device: Arc<StreamDeck>,
    assets_directory: PathBuf,
}

impl DeckRuntime {
    pub fn open(assets_directory: PathBuf) -> Result<Self, String> {
        let hid_api = new_hidapi().map_err(|error| format!("hidapi init failed: {error}"))?;
        let (device_kind, serial_number) = list_devices(&hid_api)
            .into_iter()
            .next()
            .ok_or_else(|| "no Elgato Stream Deck found".to_string())?;
        let device = StreamDeck::connect(&hid_api, device_kind, &serial_number)
            .map_err(|error| format!("stream deck connect failed: {error}"))?;

        println!(
            "connected to {:?} serial={} firmware={}",
            device_kind,
            device
                .serial_number()
                .unwrap_or_else(|_| "unknown".to_string()),
            device
                .firmware_version()
                .unwrap_or_else(|_| "unknown".to_string())
        );

        Ok(Self {
            device: Arc::new(device),
            assets_directory,
        })
    }

    pub fn apply_key_images(&self) -> Result<(), String> {
        self.device
            .set_brightness(DEFAULT_BRIGHTNESS_PERCENT)
            .map_err(format_stream_deck_error)?;
        self.device
            .clear_all_button_images()
            .map_err(format_stream_deck_error)?;

        for binding in key_bindings() {
            let image_path = self
                .assets_directory
                .join(KEY_IMAGE_DIRECTORY_NAME)
                .join(binding.image_file_name);
            let key_image = load_key_image(&image_path)?;
            self.device
                .set_button_image(binding.key_index, key_image)
                .map_err(format_stream_deck_error)?;
        }

        self.device.flush().map_err(format_stream_deck_error)?;
        Ok(())
    }

    pub fn run_event_loop_until_shutdown_or_disconnect(
        &self,
        shutdown_requested: &AtomicBool,
    ) -> Result<ConnectedSessionOutcome, String> {
        let button_reader = self.device.get_reader();

        while !shutdown_requested.load(Ordering::SeqCst) {
            let state_updates = match button_reader.read(Some(BUTTON_POLL_TIMEOUT)) {
                Ok(updates) => updates,
                Err(_) if shutdown_requested.load(Ordering::SeqCst) => {
                    return Ok(ConnectedSessionOutcome::ShutdownRequested);
                }
                Err(error) if is_interrupted_system_call(&error) => {
                    continue;
                }
                Err(error) => {
                    eprintln!("stream deck disconnected: {error}");
                    return Ok(ConnectedSessionOutcome::DeviceDisconnected);
                }
            };

            for state_update in state_updates {
                if let DeviceStateUpdate::ButtonUp(key_index) = state_update {
                    handle_button_release(key_index);
                }
            }
        }

        Ok(ConnectedSessionOutcome::ShutdownRequested)
    }

    pub fn reset_device(&self) -> Result<(), String> {
        self.device.reset().map_err(format_stream_deck_error)
    }
}

pub fn run_device_supervisor(
    project_root: PathBuf,
    shutdown_requested: &AtomicBool,
) -> Result<(), String> {
    let mut last_waiting_log_at: Option<Instant> = None;

    while !shutdown_requested.load(Ordering::SeqCst) {
        match try_open_deck_runtime(project_root.clone()) {
            Ok(deck_runtime) => {
                last_waiting_log_at = None;
                match run_connected_session(&deck_runtime, shutdown_requested)? {
                    ConnectedSessionOutcome::ShutdownRequested => {
                        if let Err(error) = deck_runtime.reset_device() {
                            eprintln!("reset failed on shutdown: {error}");
                        }
                        return Ok(());
                    }
                    ConnectedSessionOutcome::DeviceDisconnected => {
                        drop(deck_runtime);
                        sleep_while_not_shutdown(
                            DEVICE_RECONNECT_POLL_INTERVAL,
                            shutdown_requested,
                        );
                    }
                }
            }
            Err(error) => {
                log_waiting_for_device_if_due(&error, &mut last_waiting_log_at);
                sleep_while_not_shutdown(DEVICE_RECONNECT_POLL_INTERVAL, shutdown_requested);
            }
        }
    }

    Ok(())
}

fn try_open_deck_runtime(project_root: PathBuf) -> Result<DeckRuntime, String> {
    DeckRuntime::open(project_root)
}

fn run_connected_session(
    deck_runtime: &DeckRuntime,
    shutdown_requested: &AtomicBool,
) -> Result<ConnectedSessionOutcome, String> {
    deck_runtime.apply_key_images()?;
    deck_runtime.run_event_loop_until_shutdown_or_disconnect(shutdown_requested)
}

fn log_waiting_for_device_if_due(error: &str, last_waiting_log_at: &mut Option<Instant>) {
    let now = Instant::now();
    let should_log = match last_waiting_log_at {
        None => true,
        Some(previous) => now.duration_since(*previous) >= DEVICE_OPEN_RETRY_LOG_INTERVAL,
    };
    if should_log {
        println!("waiting for Stream Deck ({error})");
        *last_waiting_log_at = Some(now);
    }
}

fn sleep_while_not_shutdown(duration: Duration, shutdown_requested: &AtomicBool) {
    let wake_deadline = Instant::now() + duration;
    while Instant::now() < wake_deadline {
        if shutdown_requested.load(Ordering::SeqCst) {
            return;
        }
        let remaining = wake_deadline.saturating_duration_since(Instant::now());
        let slice = remaining.min(Duration::from_millis(100));
        if slice.is_zero() {
            return;
        }
        thread::sleep(slice);
    }
}

fn load_key_image(image_path: &Path) -> Result<DynamicImage, String> {
    image::open(image_path)
        .map_err(|error| format!("failed to open {}: {error}", image_path.display()))
}

fn handle_button_release(key_index: u8) {
    let Some(action) = action_for_key_index(key_index) else {
        return;
    };
    dispatch_action(action);
}

fn dispatch_action(action: DeckAction) {
    match action.execute() {
        Ok(()) => println!("ok: {}", action.label()),
        Err(error) => eprintln!("error on {}: {error}", action.label()),
    }
}

fn format_stream_deck_error(error: StreamDeckError) -> String {
    format!("stream deck error: {error}")
}

fn is_interrupted_system_call(error: &StreamDeckError) -> bool {
    error.to_string().contains("Interrupted system call")
}

pub fn resolve_project_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}
