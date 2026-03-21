use std::collections::HashMap;

use bevy::prelude::warn;
use dialog::history::LoreBook;
use keybinds::Keybinds;
use serde::{Deserialize, Serialize};

use crate::lore::{StoredLoreEntry, lore_book_to_stored, stored_to_lore_book};
use crate::storage;

#[derive(Debug, Default, Serialize, Deserialize)]
pub(crate) struct SaveFile {
    #[serde(default)]
    pub keybinds: HashMap<String, String>,
    #[serde(default)]
    pub lore: Vec<StoredLoreEntry>,
}

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

pub(crate) fn from_resources(keybinds: &Keybinds, lore: &LoreBook) -> SaveFile {
    SaveFile {
        keybinds: keybinds::serialize::to_map(keybinds),
        lore: lore_book_to_stored(lore),
    }
}

pub(crate) fn apply(file: SaveFile, keybinds: &mut Keybinds, lore: &mut LoreBook) {
    *keybinds = keybinds::serialize::from_map(&file.keybinds);
    *lore = stored_to_lore_book(file.lore);
}
