use crate::actions::DeckAction;

const FULL_SCREENSHOT_KEY_INDEX: u8 = 0;
const WINDOW_SCREENSHOT_KEY_INDEX: u8 = 1;
const REGION_SCREENSHOT_KEY_INDEX: u8 = 2;

const MUTE_KEY_INDEX: u8 = 5;
const UNMUTE_KEY_INDEX: u8 = 6;
const VOLUME_DOWN_KEY_INDEX: u8 = 7;
const VOLUME_UP_KEY_INDEX: u8 = 8;

const VOLUME_20_PERCENT_KEY_INDEX: u8 = 10;
const VOLUME_40_PERCENT_KEY_INDEX: u8 = 11;
const VOLUME_60_PERCENT_KEY_INDEX: u8 = 12;
const VOLUME_80_PERCENT_KEY_INDEX: u8 = 13;
const VOLUME_100_PERCENT_KEY_INDEX: u8 = 14;

const VOLUME_20_PERCENT: u8 = 20;
const VOLUME_40_PERCENT: u8 = 40;
const VOLUME_60_PERCENT: u8 = 60;
const VOLUME_80_PERCENT: u8 = 80;
const VOLUME_100_PERCENT: u8 = 100;

const FULL_SCREENSHOT_IMAGE_FILE_NAME: &str = "screenshot_full.jpg";
const WINDOW_SCREENSHOT_IMAGE_FILE_NAME: &str = "screenshot_window.jpg";
const REGION_SCREENSHOT_IMAGE_FILE_NAME: &str = "screenshot_region.jpg";

const MUTE_IMAGE_FILE_NAME: &str = "mute.jpg";
const UNMUTE_IMAGE_FILE_NAME: &str = "unmute.jpg";
const VOLUME_DOWN_IMAGE_FILE_NAME: &str = "volume_down.jpg";
const VOLUME_UP_IMAGE_FILE_NAME: &str = "volume_up.jpg";

const VOLUME_20_PERCENT_IMAGE_FILE_NAME: &str = "volume_20.jpg";
const VOLUME_40_PERCENT_IMAGE_FILE_NAME: &str = "volume_40.jpg";
const VOLUME_60_PERCENT_IMAGE_FILE_NAME: &str = "volume_60.jpg";
const VOLUME_80_PERCENT_IMAGE_FILE_NAME: &str = "volume_80.jpg";
const VOLUME_100_PERCENT_IMAGE_FILE_NAME: &str = "volume_100.jpg";

#[derive(Clone, Copy)]
pub struct KeyBinding {
    pub key_index: u8,
    pub action: DeckAction,
    pub image_file_name: &'static str,
}

pub fn key_bindings() -> &'static [KeyBinding] {
    &[
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
            key_index: VOLUME_20_PERCENT_KEY_INDEX,
            action: DeckAction::SetVolumePercent(VOLUME_20_PERCENT),
            image_file_name: VOLUME_20_PERCENT_IMAGE_FILE_NAME,
        },
        KeyBinding {
            key_index: VOLUME_40_PERCENT_KEY_INDEX,
            action: DeckAction::SetVolumePercent(VOLUME_40_PERCENT),
            image_file_name: VOLUME_40_PERCENT_IMAGE_FILE_NAME,
        },
        KeyBinding {
            key_index: VOLUME_60_PERCENT_KEY_INDEX,
            action: DeckAction::SetVolumePercent(VOLUME_60_PERCENT),
            image_file_name: VOLUME_60_PERCENT_IMAGE_FILE_NAME,
        },
        KeyBinding {
            key_index: VOLUME_80_PERCENT_KEY_INDEX,
            action: DeckAction::SetVolumePercent(VOLUME_80_PERCENT),
            image_file_name: VOLUME_80_PERCENT_IMAGE_FILE_NAME,
        },
        KeyBinding {
            key_index: VOLUME_100_PERCENT_KEY_INDEX,
            action: DeckAction::SetVolumePercent(VOLUME_100_PERCENT),
            image_file_name: VOLUME_100_PERCENT_IMAGE_FILE_NAME,
        },
    ]
}

pub fn action_for_key_index(key_index: u8) -> Option<DeckAction> {
    key_bindings()
        .iter()
        .find(|binding| binding.key_index == key_index)
        .map(|binding| binding.action)
}
