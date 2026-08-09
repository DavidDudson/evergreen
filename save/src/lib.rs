//! Unified save crate -- owns all platform I/O for persistent state.
//!
//! See [`Persistable`] for how to add a new persisted resource and
//! [`SavePlugin`] for the wiring entry point.

mod file;
mod persistable;
mod plugin;
mod storage;

pub use file::{SaveFile, SAVE_VERSION};
pub use persistable::{Migrator, Persistable, PersistableAppExt};
pub use plugin::SavePlugin;
pub use storage::{NativeBackend, StorageBackend, WasmBackend};
