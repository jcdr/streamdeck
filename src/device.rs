use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

use elgato_streamdeck::{
    list_devices, new_hidapi, DeviceStateUpdate, StreamDeck, StreamDeckError,
};
use image::{DynamicImage, Rgba};

use crate::actions::DeckAction;
use crate::clock_display::{
    render_date_key_image, render_time_key_image, ClockSnapshot, DATE_KEY_INDEX, TIME_KEY_INDEX,
};
use crate::keys::{action_for_key_index, key_bindings};

const KEY_IMAGE_DIRECTORY_NAME: &str = "assets/keys";
const DEFAULT_BRIGHTNESS_PERCENT: u8 = 70;
const BUTTON_POLL_TIMEOUT: Duration = Duration::from_millis(100);
const DEVICE_RECONNECT_POLL_INTERVAL: Duration = Duration::from_secs(2);
const DEVICE_OPEN_RETRY_LOG_INTERVAL: Duration = Duration::from_secs(30);
const KEY_PRESS_FLASH_BLEND_NUMERATOR: u16 = 7;
const KEY_PRESS_FLASH_BLEND_DENOMINATOR: u16 = 10;
const KEY_PRESS_FLASH_WHITE_LEVEL: u16 = 255;

pub enum ConnectedSessionOutcome {
    ShutdownRequested,
    DeviceDisconnected,
}

struct KeyVisualState {
    idle_image: DynamicImage,
    pressed_image: DynamicImage,
}

pub struct DeckRuntime {
    device: Arc<StreamDeck>,
    assets_directory: PathBuf,
    key_visual_states: HashMap<u8, KeyVisualState>,
    last_clock_snapshot: Option<ClockSnapshot>,
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
            key_visual_states: HashMap::new(),
            last_clock_snapshot: None,
        })
    }

    pub fn apply_key_images(&mut self) -> Result<(), String> {
        self.device
            .set_brightness(DEFAULT_BRIGHTNESS_PERCENT)
            .map_err(format_stream_deck_error)?;
        self.device
            .clear_all_button_images()
            .map_err(format_stream_deck_error)?;

        self.key_visual_states.clear();
        self.last_clock_snapshot = None;

        for binding in key_bindings() {
            let image_path = self
                .assets_directory
                .join(KEY_IMAGE_DIRECTORY_NAME)
                .join(binding.image_file_name);
            let idle_image = load_key_image(&image_path)?;
            self.install_key_visual(binding.key_index, idle_image)?;
        }

        self.refresh_clock_keys(true)?;
        self.device.flush().map_err(format_stream_deck_error)?;
        Ok(())
    }

    pub fn run_event_loop_until_shutdown_or_disconnect(
        &mut self,
        shutdown_requested: &AtomicBool,
    ) -> Result<ConnectedSessionOutcome, String> {
        let button_reader = self.device.get_reader();

        while !shutdown_requested.load(Ordering::SeqCst) {
            if let Err(error) = self.refresh_clock_keys(false) {
                eprintln!("clock refresh failed: {error}");
            }

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
                match state_update {
                    DeviceStateUpdate::ButtonDown(key_index) => {
                        self.apply_key_press_effect(key_index);
                    }
                    DeviceStateUpdate::ButtonUp(key_index) => {
                        self.apply_key_release_effect(key_index);
                        handle_button_release(key_index);
                    }
                    _ => {}
                }
            }
        }

        Ok(ConnectedSessionOutcome::ShutdownRequested)
    }

    pub fn reset_device(&self) -> Result<(), String> {
        self.device.reset().map_err(format_stream_deck_error)
    }

    fn refresh_clock_keys(&mut self, force_redraw: bool) -> Result<(), String> {
        let snapshot = ClockSnapshot::from_local_now();
        if !force_redraw && self.last_clock_snapshot.as_ref() == Some(&snapshot) {
            return Ok(());
        }

        let date_changed = self
            .last_clock_snapshot
            .as_ref()
            .map(|previous| {
                previous.date_year_line != snapshot.date_year_line
                    || previous.date_month_line != snapshot.date_month_line
                    || previous.date_day_line != snapshot.date_day_line
            })
            .unwrap_or(true);

        if force_redraw || date_changed {
            let date_image = render_date_key_image(&snapshot)?;
            self.install_key_visual(DATE_KEY_INDEX, date_image)?;
        }

        let time_image = render_time_key_image(&snapshot)?;
        self.install_key_visual(TIME_KEY_INDEX, time_image)?;

        self.last_clock_snapshot = Some(snapshot);
        self.device.flush().map_err(format_stream_deck_error)
    }

    fn install_key_visual(
        &mut self,
        key_index: u8,
        idle_image: DynamicImage,
    ) -> Result<(), String> {
        let pressed_image = create_pressed_key_image(&idle_image);
        self.device
            .set_button_image(key_index, idle_image.clone())
            .map_err(format_stream_deck_error)?;
        self.key_visual_states.insert(
            key_index,
            KeyVisualState {
                idle_image,
                pressed_image,
            },
        );
        Ok(())
    }

    fn apply_key_press_effect(&self, key_index: u8) {
        let Some(visual_state) = self.key_visual_states.get(&key_index) else {
            return;
        };
        if let Err(error) = self.write_key_image(key_index, visual_state.pressed_image.clone()) {
            eprintln!("press effect failed on key {key_index}: {error}");
        }
    }

    fn apply_key_release_effect(&self, key_index: u8) {
        let Some(visual_state) = self.key_visual_states.get(&key_index) else {
            return;
        };
        if let Err(error) = self.write_key_image(key_index, visual_state.idle_image.clone()) {
            eprintln!("release effect failed on key {key_index}: {error}");
        }
    }

    fn write_key_image(&self, key_index: u8, image: DynamicImage) -> Result<(), String> {
        self.device
            .set_button_image(key_index, image)
            .map_err(format_stream_deck_error)?;
        self.device.flush().map_err(format_stream_deck_error)
    }
}

pub fn run_device_supervisor(
    project_root: PathBuf,
    shutdown_requested: &AtomicBool,
) -> Result<(), String> {
    let mut last_waiting_log_at: Option<Instant> = None;

    while !shutdown_requested.load(Ordering::SeqCst) {
        match try_open_deck_runtime(project_root.clone()) {
            Ok(mut deck_runtime) => {
                last_waiting_log_at = None;
                match run_connected_session(&mut deck_runtime, shutdown_requested)? {
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
    deck_runtime: &mut DeckRuntime,
    shutdown_requested: &AtomicBool,
) -> Result<ConnectedSessionOutcome, String> {
    deck_runtime.apply_key_images()?;
    deck_runtime.run_event_loop_until_shutdown_or_disconnect(shutdown_requested)
}

fn create_pressed_key_image(idle_image: &DynamicImage) -> DynamicImage {
    let mut rgba_image = idle_image.to_rgba8();
    for pixel in rgba_image.pixels_mut() {
        *pixel = blend_pixel_toward_white(*pixel);
    }
    DynamicImage::ImageRgba8(rgba_image)
}

fn blend_pixel_toward_white(pixel: Rgba<u8>) -> Rgba<u8> {
    let red = blend_channel_toward_white(pixel[0]);
    let green = blend_channel_toward_white(pixel[1]);
    let blue = blend_channel_toward_white(pixel[2]);
    Rgba([red, green, blue, pixel[3]])
}

fn blend_channel_toward_white(channel: u8) -> u8 {
    let blended = (u16::from(channel)
        * (KEY_PRESS_FLASH_BLEND_DENOMINATOR - KEY_PRESS_FLASH_BLEND_NUMERATOR)
        + KEY_PRESS_FLASH_WHITE_LEVEL * KEY_PRESS_FLASH_BLEND_NUMERATOR)
        / KEY_PRESS_FLASH_BLEND_DENOMINATOR;
    blended as u8
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
