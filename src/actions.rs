use crate::audio;
use crate::file_browser;
use crate::screenshot;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DeckAction {
    MuteAudio,
    UnmuteAudio,
    IncreaseVolume,
    DecreaseVolume,
    SetVolumePercent(u8),
    FullScreenshot,
    WindowScreenshot,
    RegionScreenshot,
    OpenScreenshotsFolder,
    IncreaseDeckBrightness,
    DecreaseDeckBrightness,
}

impl DeckAction {
    pub fn execute(self) -> Result<(), String> {
        match self {
            DeckAction::MuteAudio => audio::mute_default_sink(),
            DeckAction::UnmuteAudio => audio::unmute_default_sink(),
            DeckAction::IncreaseVolume => audio::increase_default_sink_volume(),
            DeckAction::DecreaseVolume => audio::decrease_default_sink_volume(),
            DeckAction::SetVolumePercent(volume_percent) => {
                audio::set_default_sink_volume_percent(volume_percent)
            }
            DeckAction::FullScreenshot => screenshot::capture_full_screenshot(),
            DeckAction::WindowScreenshot => screenshot::capture_active_window_screenshot(),
            DeckAction::RegionScreenshot => screenshot::capture_region_screenshot(),
            DeckAction::OpenScreenshotsFolder => file_browser::open_screenshots_directory(),
            DeckAction::IncreaseDeckBrightness | DeckAction::DecreaseDeckBrightness => {
                Err("deck brightness is applied by the device runtime".to_string())
            }
        }
    }

    pub fn label(self) -> String {
        match self {
            DeckAction::MuteAudio => "mute audio".to_string(),
            DeckAction::UnmuteAudio => "unmute audio".to_string(),
            DeckAction::IncreaseVolume => "increase volume".to_string(),
            DeckAction::DecreaseVolume => "decrease volume".to_string(),
            DeckAction::SetVolumePercent(volume_percent) => {
                format!("set volume to {volume_percent}%")
            }
            DeckAction::FullScreenshot => "full screenshot".to_string(),
            DeckAction::WindowScreenshot => "window screenshot".to_string(),
            DeckAction::RegionScreenshot => "region screenshot".to_string(),
            DeckAction::OpenScreenshotsFolder => "open screenshots folder".to_string(),
            DeckAction::IncreaseDeckBrightness => "increase deck brightness 10%".to_string(),
            DeckAction::DecreaseDeckBrightness => "decrease deck brightness 10%".to_string(),
        }
    }
}
