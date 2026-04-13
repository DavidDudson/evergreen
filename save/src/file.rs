use std::collections::HashMap;

use bevy::prelude::warn;
use dialog::history::LoreBook;
use keybinds::Keybinds;
use models::settings::GameSettings;
use serde::{Deserialize, Serialize};

use crate::lore::{lore_book_to_stored, stored_to_lore_book, StoredLoreEntry};
use crate::storage;

// ---------------------------------------------------------------------------
// Stored types
// ---------------------------------------------------------------------------

#[derive(Debug, Default, Serialize, Deserialize)]
pub(crate) struct SaveFile {
    #[serde(default)]
    pub keybinds: HashMap<String, String>,
    #[serde(default)]
    pub lore: Vec<StoredLoreEntry>,
    #[serde(default)]
    pub settings: StoredSettings,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct StoredSettings {
    #[serde(default = "default_ten")]
    pub master_volume: u8,
    #[serde(default = "default_eight")]
    pub bgm_volume: u8,
    #[serde(default = "default_ten")]
    pub sfx_volume: u8,
    #[serde(default)]
    pub fullscreen: bool,
    #[serde(default = "default_language")]
    pub language: String,
}

impl Default for StoredSettings {
    fn default() -> Self {
        let d = GameSettings::default();
        Self {
            master_volume: d.master_volume,
            bgm_volume: d.bgm_volume,
            sfx_volume: d.sfx_volume,
            fullscreen: d.fullscreen,
            language: d.language,
        }
    }
}

fn default_ten() -> u8 {
    10
}
fn default_eight() -> u8 {
    8
}
fn default_language() -> String {
    "en-US".to_owned()
}

// ---------------------------------------------------------------------------
// Serialization helpers
// ---------------------------------------------------------------------------

pub(crate) fn load() -> Option<SaveFile> {
    let json = storage::read()?;
    serde_json::from_str(&json)
        .map_err(|e| warn!("Save file corrupt, using defaults: {e}"))
        .ok()
}

pub(crate) fn persist(file: &SaveFile) {
    match serde_json::to_string(file) {
        Ok(json) => {
            if let Err(e) = storage::write(&json) {
                warn!("Failed to write save file: {e}");
            }
        }
        Err(e) => warn!("Failed to serialize save file: {e}"),
    }
}

pub(crate) fn from_resources(
    keybinds: &Keybinds,
    lore: &LoreBook,
    settings: &GameSettings,
) -> SaveFile {
    SaveFile {
        keybinds: keybinds::serialize::to_map(keybinds),
        lore: lore_book_to_stored(lore),
        settings: StoredSettings {
            master_volume: settings.master_volume,
            bgm_volume: settings.bgm_volume,
            sfx_volume: settings.sfx_volume,
            fullscreen: settings.fullscreen,
            language: settings.language.clone(),
        },
    }
}

pub(crate) fn apply(
    file: SaveFile,
    keybinds: &mut Keybinds,
    lore: &mut LoreBook,
    settings: &mut GameSettings,
) {
    *keybinds = keybinds::serialize::from_map(&file.keybinds);
    *lore = stored_to_lore_book(file.lore);
    settings.master_volume = file.settings.master_volume;
    settings.bgm_volume = file.settings.bgm_volume;
    settings.sfx_volume = file.settings.sfx_volume;
    settings.fullscreen = file.settings.fullscreen;
    settings.language = file.settings.language;
}
