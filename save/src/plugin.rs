//! Wires the unified save lifecycle into the Bevy app.
//!
//! Owns the cfg gate that selects [`WasmBackend`] vs [`NativeBackend`] and
//! holds the orphan-rule [`Persistable`] impls for the foreign resources
//! (`Keybinds`, `LoreBook`, `GameSettings`) that the project persists.

use bevy::prelude::*;
use dialog::history::LoreBook;
use keybinds::Keybinds;
use models::multiverse::MultiverseSave;
use models::settings::GameSettings;

use crate::file::SaveFile;
use crate::persistable::{load_save_file, PersistableAppExt};
use crate::Persistable;

pub struct SavePlugin;

impl Plugin for SavePlugin {
    fn build(&self, app: &mut App) {
        // Single cfg gate: pick the storage backend for the active target.
        #[cfg(target_arch = "wasm32")]
        app.insert_resource(crate::storage::WasmBackend);
        #[cfg(not(target_arch = "wasm32"))]
        app.insert_resource(crate::storage::NativeBackend);

        // Make sure GameSettings exists -- other crates (e.g. dialog) read it
        // during their own Startup before persistable load runs.
        app.init_resource::<GameSettings>();

        // Envelope: load before any per-slot loads.
        app.init_resource::<SaveFile>()
            .add_systems(PreStartup, load_save_file);

        // Per-resource slots.
        app.register_persistable::<Keybinds>()
            .register_persistable::<LoreBook>()
            .register_persistable::<GameSettings>()
            .register_persistable::<MultiverseSave>();
    }
}

// ---------------------------------------------------------------------------
// Persistable impls (orphan rule: these foreign types are persisted here).
// ---------------------------------------------------------------------------

impl Persistable for Keybinds {
    const KEY: &'static str = "keybinds";
}

impl Persistable for LoreBook {
    const KEY: &'static str = "lore";
}

impl Persistable for GameSettings {
    const KEY: &'static str = "settings";
}

impl Persistable for MultiverseSave {
    const KEY: &'static str = "multiverse";
}
