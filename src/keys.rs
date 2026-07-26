use crate::actions::DeckAction;

const MUTE_KEY_INDEX: u8 = 0;
const UNMUTE_KEY_INDEX: u8 = 1;
const VOLUME_DOWN_KEY_INDEX: u8 = 2;
const VOLUME_UP_KEY_INDEX: u8 = 3;
const FULL_SCREENSHOT_KEY_INDEX: u8 = 5;
const WINDOW_SCREENSHOT_KEY_INDEX: u8 = 6;
const REGION_SCREENSHOT_KEY_INDEX: u8 = 7;

const MUTE_IMAGE_FILE_NAME: &str = "mute.jpg";
const UNMUTE_IMAGE_FILE_NAME: &str = "unmute.jpg";
const VOLUME_DOWN_IMAGE_FILE_NAME: &str = "volume_down.jpg";
const VOLUME_UP_IMAGE_FILE_NAME: &str = "volume_up.jpg";
const FULL_SCREENSHOT_IMAGE_FILE_NAME: &str = "screenshot_full.jpg";
const WINDOW_SCREENSHOT_IMAGE_FILE_NAME: &str = "screenshot_window.jpg";
const REGION_SCREENSHOT_IMAGE_FILE_NAME: &str = "screenshot_region.jpg";

#[derive(Clone, Copy)]
pub struct KeyBinding {
    pub key_index: u8,
    pub action: DeckAction,
    pub image_file_name: &'static str,
}

pub fn key_bindings() -> &'static [KeyBinding] {
    &[
        KeyBinding {
            key_index: MUTE_KEY_INDEX,
            action: DeckAction::MuteAudio,
            image_file_name: MUTE_IMAGE_FILE_NAME,
        },
        KeyBinding {
            key_index: UNMUTE_KEY_INDEX,
            action: DeckAction::UnmuteAudio,
            image_file_name: UNMUTE_IMAGE_FILE_NAME,
        },
        KeyBinding {
            key_index: VOLUME_DOWN_KEY_INDEX,
            action: DeckAction::DecreaseVolume,
            image_file_name: VOLUME_DOWN_IMAGE_FILE_NAME,
        },
        KeyBinding {
            key_index: VOLUME_UP_KEY_INDEX,
            action: DeckAction::IncreaseVolume,
            image_file_name: VOLUME_UP_IMAGE_FILE_NAME,
        },
        KeyBinding {
            key_index: FULL_SCREENSHOT_KEY_INDEX,
            action: DeckAction::FullScreenshot,
            image_file_name: FULL_SCREENSHOT_IMAGE_FILE_NAME,
        },
        KeyBinding {
            key_index: WINDOW_SCREENSHOT_KEY_INDEX,
            action: DeckAction::WindowScreenshot,
            image_file_name: WINDOW_SCREENSHOT_IMAGE_FILE_NAME,
        },
        KeyBinding {
            key_index: REGION_SCREENSHOT_KEY_INDEX,
            action: DeckAction::RegionScreenshot,
            image_file_name: REGION_SCREENSHOT_IMAGE_FILE_NAME,
        },
    ]
}

pub fn action_for_key_index(key_index: u8) -> Option<DeckAction> {
    key_bindings()
        .iter()
        .find(|binding| binding.key_index == key_index)
        .map(|binding| binding.action)
}
