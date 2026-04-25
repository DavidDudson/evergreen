//! Generic persistence trait + extension method for plugging resources into
//! the unified save file.
//!
//! A [`Persistable`] resource declares a stable [`KEY`](Persistable::KEY) and
//! gets two systems registered automatically by
//! [`PersistableAppExt::register_persistable`]:
//!
//! - `load_slot::<T>` (PreStartup, after `load_save_file`) -- decodes the slot
//!   from [`SaveFile`] into the resource, falling back to `T::default()` if
//!   the slot is missing or malformed.
//! - `save_slot::<T>` (PostUpdate, on `resource_changed::<T>`) -- re-encodes
//!   the resource into [`SaveFile`] and writes the envelope through the
//!   active [`StorageBackend`].

use bevy::prelude::*;
use serde::{de::DeserializeOwned, Serialize};

use crate::file::{SaveFile, SAVE_VERSION};
use crate::storage::StorageBackend;

/// A resource that can be saved to and loaded from a slot in [`SaveFile`].
///
/// Implementors must be uniquely keyed (`KEY`) and have a sensible `Default`
/// so missing/corrupt slots can fall back without crashing.
pub trait Persistable: Resource + Serialize + DeserializeOwned + Default {
    /// Stable key used as the slot name in [`SaveFile::slots`].
    /// Must be unique across all `Persistable` impls.
    const KEY: &'static str;
}

/// Future hook for slot-level migrations between schema versions.
///
/// Not yet wired into [`PersistableAppExt::register_persistable`]; see the
/// `version` field on [`SaveFile`] and [`SAVE_VERSION`] for the envelope-level
/// counterpart.
pub trait Migrator {
    /// The resource type this migrator targets.
    type Target: Persistable;
    /// Attempt to upgrade a stored slot value from `from_version` to the
    /// current version.
    fn migrate(value: serde_json::Value, from_version: u32) -> Option<serde_json::Value>;
}

/// `App` extension that wires a [`Persistable`] into the load/save lifecycle.
pub trait PersistableAppExt {
    /// Registers `T` as a persistable resource.
    ///
    /// Inserts `T::default()` if not already present, then schedules
    /// `load_slot::<T>` in `PreStartup` (after [`load_save_file`]) and
    /// `save_slot::<T>` in `PostUpdate` (gated on `resource_changed::<T>`).
    fn register_persistable<T: Persistable>(&mut self) -> &mut Self;
}

impl PersistableAppExt for App {
    fn register_persistable<T: Persistable>(&mut self) -> &mut Self {
        self.init_resource::<T>()
            .add_systems(PreStartup, load_slot::<T>.after(load_save_file))
            .add_systems(
                PostUpdate,
                save_slot::<T, WasmOrNativeBackend>.run_if(resource_changed::<T>),
            )
    }
}

// The save_slot system needs a concrete backend type at registration time.
// `WasmOrNativeBackend` is a typedef chosen by `cfg`; the plugin inserts the
// matching `Resource` impl exactly once.
#[cfg(target_arch = "wasm32")]
pub(crate) type WasmOrNativeBackend = crate::storage::WasmBackend;
#[cfg(not(target_arch = "wasm32"))]
pub(crate) type WasmOrNativeBackend = crate::storage::NativeBackend;

// ---------------------------------------------------------------------------
// PreStartup: load envelope from disk/localStorage into `SaveFile` resource.
// ---------------------------------------------------------------------------

/// Reads the persisted envelope (if any) and inserts it as a `SaveFile`
/// resource. Runs at `PreStartup` order 0 so all `load_slot::<T>` systems
/// (registered with `.after(load_save_file)`) see a populated envelope.
pub fn load_save_file(mut commands: Commands, backend: Res<WasmOrNativeBackend>) {
    let file = backend
        .read()
        .and_then(|raw| {
            serde_json::from_str::<SaveFile>(&raw)
                .map_err(|e| warn!("Save file corrupt, using defaults: {e}"))
                .ok()
        })
        .inspect(|loaded| {
            if loaded.version != SAVE_VERSION {
                warn!(
                    "Save file version mismatch: found {}, expected {}. Slots will be \
                     loaded best-effort and missing fields filled from defaults.",
                    loaded.version, SAVE_VERSION
                );
            }
        })
        .unwrap_or_default();

    commands.insert_resource(file);
}

// ---------------------------------------------------------------------------
// PreStartup (per resource): decode slot into resource.
// ---------------------------------------------------------------------------

fn load_slot<T: Persistable>(save: Res<SaveFile>, mut target: ResMut<T>) {
    let Some(raw) = save.slots.get(T::KEY) else {
        return;
    };
    match serde_json::from_value::<T>(raw.clone()) {
        Ok(decoded) => *target = decoded,
        Err(e) => warn!(
            "Save slot '{}' could not be decoded ({e}); keeping default.",
            T::KEY
        ),
    }
}

// ---------------------------------------------------------------------------
// PostUpdate (per resource): encode resource and persist envelope.
// ---------------------------------------------------------------------------

fn save_slot<T: Persistable, B: StorageBackend>(
    source: Res<T>,
    mut save: ResMut<SaveFile>,
    backend: Res<B>,
) {
    match serde_json::to_value(&*source) {
        Ok(value) => {
            save.slots.insert(T::KEY.to_owned(), value);
            save.version = SAVE_VERSION;
        }
        Err(e) => {
            warn!("Failed to encode save slot '{}': {e}", T::KEY);
            return;
        }
    }

    match serde_json::to_string(&*save) {
        Ok(json) => backend.write(&json),
        Err(e) => warn!("Failed to serialize save envelope: {e}"),
    }
}
