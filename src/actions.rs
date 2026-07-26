use crate::audio;
use crate::screenshot;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DeckAction {
    MuteAudio,
    UnmuteAudio,
    IncreaseVolume,
    DecreaseVolume,
    FullScreenshot,
    WindowScreenshot,
    RegionScreenshot,
}

impl DeckAction {
    pub fn execute(self) -> Result<(), String> {
        match self {
            DeckAction::MuteAudio => audio::mute_default_sink(),
            DeckAction::UnmuteAudio => audio::unmute_default_sink(),
            DeckAction::IncreaseVolume => audio::increase_default_sink_volume(),
            DeckAction::DecreaseVolume => audio::decrease_default_sink_volume(),
            DeckAction::FullScreenshot => screenshot::capture_full_screenshot(),
            DeckAction::WindowScreenshot => screenshot::capture_active_window_screenshot(),
            DeckAction::RegionScreenshot => screenshot::capture_region_screenshot(),
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            DeckAction::MuteAudio => "mute audio",
            DeckAction::UnmuteAudio => "unmute audio",
            DeckAction::IncreaseVolume => "increase volume",
            DeckAction::DecreaseVolume => "decrease volume",
            DeckAction::FullScreenshot => "full screenshot",
            DeckAction::WindowScreenshot => "window screenshot",
            DeckAction::RegionScreenshot => "region screenshot",
        }
    }
}
