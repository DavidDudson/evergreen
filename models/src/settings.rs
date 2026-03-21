use bevy::prelude::Resource;

/// Persistent user preferences: audio volumes and display options.
///
/// Stored by the `save` crate; apply side-effects (window mode, audio bus
/// gain) via the sync systems in `ui`.
#[derive(Resource, Debug, Clone)]
pub struct GameSettings {
    /// Master volume 0–10.
    pub master_volume: u8,
    /// Background music volume 0–10.
    pub bgm_volume: u8,
    /// Sound effects volume 0–10.
    pub sfx_volume: u8,
    /// Whether the window is in borderless-fullscreen mode.
    pub fullscreen: bool,
    /// Active locale code, e.g. `"en-US"`. Must match a file in `assets/locale/`.
    pub language: String,
}

impl Default for GameSettings {
    fn default() -> Self {
        Self {
            master_volume: 10,
            bgm_volume: 8,
            sfx_volume: 10,
            fullscreen: false,
            language: "en-US".to_owned(),
        }
    }
}
