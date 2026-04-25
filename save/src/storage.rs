//! Platform-specific storage backends for the unified save file.
//!
//! Implementations:
//! - [`WasmBackend`] reads/writes `localStorage["evergreen.save"]`
//! - [`NativeBackend`] reads/writes `./evergreen_saves/evergreen.save.json`
//!
//! [`SavePlugin`](crate::SavePlugin) selects the appropriate backend at insert
//! time based on `cfg(target_arch = "wasm32")`. Systems use the
//! [`StorageBackend`] trait so they remain platform-agnostic.

use bevy::prelude::Resource;

pub(crate) const SAVE_KEY: &str = "evergreen.save";

/// Trait abstracting persistent storage so save systems can be platform-agnostic.
pub trait StorageBackend: Resource {
    /// Reads the current save blob, or `None` if no save exists or read failed.
    fn read(&self) -> Option<String>;
    /// Writes the save blob, logging any failure (does not return an error).
    fn write(&self, content: &str);
}

// ---------------------------------------------------------------------------
// WASM (browser localStorage)
// ---------------------------------------------------------------------------

/// Stores the save blob in browser `localStorage`.
#[derive(Resource, Default)]
pub struct WasmBackend;

#[cfg(target_arch = "wasm32")]
impl StorageBackend for WasmBackend {
    fn read(&self) -> Option<String> {
        use web_sys::window;
        window()?.local_storage().ok()??.get_item(SAVE_KEY).ok()?
    }

    fn write(&self, content: &str) {
        use bevy::prelude::warn;
        use wasm_bindgen::JsValue;
        use web_sys::window;

        let result = window()
            .ok_or_else(|| "no window".to_owned())
            .and_then(|w| {
                w.local_storage()
                    .map_err(|e| format!("{e:?}"))?
                    .ok_or_else(|| "no localStorage".to_owned())
            })
            .and_then(|storage| {
                storage
                    .set_item(SAVE_KEY, content)
                    .map_err(|e: JsValue| format!("{e:?}"))
            });

        if let Err(e) = result {
            warn!("Failed to write save to localStorage: {e}");
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl StorageBackend for WasmBackend {
    fn read(&self) -> Option<String> {
        None
    }
    fn write(&self, _content: &str) {}
}

// ---------------------------------------------------------------------------
// Native (filesystem)
// ---------------------------------------------------------------------------

/// Stores the save blob at `./evergreen_saves/evergreen.save.json`.
#[derive(Resource, Default)]
pub struct NativeBackend;

#[cfg(not(target_arch = "wasm32"))]
const NATIVE_SAVE_PATH: &str = "./evergreen_saves/evergreen.save.json";

#[cfg(not(target_arch = "wasm32"))]
impl StorageBackend for NativeBackend {
    fn read(&self) -> Option<String> {
        std::fs::read_to_string(NATIVE_SAVE_PATH).ok()
    }

    fn write(&self, content: &str) {
        use bevy::prelude::warn;
        use std::path::Path;
        let path = Path::new(NATIVE_SAVE_PATH);
        let result = path
            .parent()
            .ok_or_else(|| "save path has no parent".to_owned())
            .and_then(|parent| std::fs::create_dir_all(parent).map_err(|e| e.to_string()))
            .and_then(|()| std::fs::write(path, content).map_err(|e| e.to_string()));

        if let Err(e) = result {
            warn!("Failed to write save file: {e}");
        }
    }
}

#[cfg(target_arch = "wasm32")]
impl StorageBackend for NativeBackend {
    fn read(&self) -> Option<String> {
        None
    }
    fn write(&self, _content: &str) {}
}
