use bevy::prelude::*;

use crate::bindings::Keybinds;
use crate::serialize;

const STORAGE_KEY: &str = "evergreen.keybinds";

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// System: save [`Keybinds`] to persistent storage whenever it changes.
pub fn save_on_change(keybinds: Res<Keybinds>) {
    if !keybinds.is_changed() {
        return;
    }
    if let Err(e) = save(&keybinds) {
        warn!("Failed to save keybinds: {e}");
    }
}

/// Loads saved keybinds from persistent storage, merging over defaults.
/// Returns `None` if nothing has been saved yet.
pub fn load() -> Option<Keybinds> {
    let json = read_storage(STORAGE_KEY)?;
    let result = serialize::from_json(&json);
    if result.is_none() {
        warn!("Keybind storage corrupt or unrecognized, using defaults");
    }
    result
}

// ---------------------------------------------------------------------------
// Platform implementations
// ---------------------------------------------------------------------------

fn save(keybinds: &Keybinds) -> Result<(), String> {
    let json = serialize::to_json(keybinds);
    write_storage(STORAGE_KEY, &json)
}

// --- WASM -------------------------------------------------------------------

#[cfg(target_arch = "wasm32")]
fn write_storage(key: &str, value: &str) -> Result<(), String> {
    use wasm_bindgen::JsValue;
    use web_sys::window;

    let storage = window()
        .ok_or("no window")?
        .local_storage()
        .map_err(|e| format!("{e:?}"))?
        .ok_or("no localStorage")?;

    storage
        .set_item(key, value)
        .map_err(|e: JsValue| format!("{e:?}"))
}

#[cfg(target_arch = "wasm32")]
fn read_storage(key: &str) -> Option<String> {
    use web_sys::window;

    window()?
        .local_storage()
        .ok()??
        .get_item(key)
        .ok()?
}

// --- Native (dev / test) ----------------------------------------------------

#[cfg(not(target_arch = "wasm32"))]
fn write_storage(key: &str, value: &str) -> Result<(), String> {
    use std::fs;
    let path = native_path(key);
    fs::create_dir_all(path.parent().expect("path has parent"))
        .map_err(|e| e.to_string())?;
    fs::write(&path, value).map_err(|e| e.to_string())
}

#[cfg(not(target_arch = "wasm32"))]
fn read_storage(key: &str) -> Option<String> {
    std::fs::read_to_string(native_path(key)).ok()
}

#[cfg(not(target_arch = "wasm32"))]
fn native_path(key: &str) -> std::path::PathBuf {
    std::path::PathBuf::from(format!("./evergreen_saves/{key}.json"))
}
