use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use elgato_streamdeck::{
    list_devices, new_hidapi, DeviceStateUpdate, StreamDeck, StreamDeckError,
};
use image::DynamicImage;

use crate::actions::DeckAction;
use crate::keys::{action_for_key_index, key_bindings};

const KEY_IMAGE_DIRECTORY_NAME: &str = "assets/keys";
const DEFAULT_BRIGHTNESS_PERCENT: u8 = 70;
const BUTTON_POLL_TIMEOUT: Duration = Duration::from_millis(100);

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

    pub fn run_event_loop_until_shutdown(
        &self,
        shutdown_requested: &AtomicBool,
    ) -> Result<(), String> {
        let button_reader = self.device.get_reader();

        while !shutdown_requested.load(Ordering::SeqCst) {
            let state_updates = match button_reader.read(Some(BUTTON_POLL_TIMEOUT)) {
                Ok(updates) => updates,
                Err(_) if shutdown_requested.load(Ordering::SeqCst) => {
                    break;
                }
                Err(error) if is_interrupted_system_call(&error) => {
                    continue;
                }
                Err(error) => {
                    return Err(format!("stream deck read failed: {error}"));
                }
            };

            for state_update in state_updates {
                if let DeviceStateUpdate::ButtonUp(key_index) = state_update {
                    handle_button_release(key_index);
                }
            }
        }

        Ok(())
    }

    pub fn reset_device(&self) -> Result<(), String> {
        self.device.reset().map_err(format_stream_deck_error)
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
