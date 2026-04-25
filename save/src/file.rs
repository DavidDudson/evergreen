//! Unified save file: a versioned envelope around a map of typed slots.
//!
//! Each [`Persistable`](crate::Persistable) resource owns one slot keyed by
//! its `KEY` constant. Slot values are stored as `serde_json::Value` so the
//! envelope itself never depends on the concrete resource types.

use std::collections::HashMap;

use bevy::prelude::Resource;
use serde::{Deserialize, Serialize};

/// Current on-disk schema version. Bump when the envelope format itself
/// changes (slot ownership, key conventions, etc.). Slot-internal migrations
/// are the responsibility of each `Persistable`'s [`Migrator`](crate::Migrator).
pub const SAVE_VERSION: u32 = 2;

/// Top-level save envelope. Stored as a single JSON document.
#[derive(Resource, Debug, Default, Serialize, Deserialize)]
pub struct SaveFile {
    /// Schema version of this envelope.
    #[serde(default)]
    pub version: u32,
    /// Per-resource slots keyed by [`Persistable::KEY`](crate::Persistable::KEY).
    #[serde(default)]
    pub slots: HashMap<String, serde_json::Value>,
}
