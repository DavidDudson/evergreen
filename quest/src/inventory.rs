//! Player inventory: a flat string-keyed bag of items with stack counts.
//!
//! Items are referenced by stable `&'static str` IDs (e.g.
//! [`item::MIRROR_SHARD`]) so quest scripts and code share a single namespace
//! without a parallel enum to maintain.

use std::collections::HashMap;

use bevy::prelude::Resource;
use serde::{Deserialize, Serialize};

/// Persisted player inventory. Stack counts default to zero for absent items.
#[derive(Resource, Debug, Default, Clone, Serialize, Deserialize)]
pub struct Inventory {
    pub items: HashMap<String, u32>,
}

impl Inventory {
    /// Add `count` copies of `item` to the bag.
    pub fn grant(&mut self, item: impl Into<String>, count: u32) {
        let entry = self.items.entry(item.into()).or_insert(0);
        *entry = entry.saturating_add(count);
    }

    pub fn count(&self, item: &str) -> u32 {
        self.items.get(item).copied().unwrap_or(0)
    }

    pub fn has(&self, item: &str) -> bool {
        self.count(item) > 0
    }
}

/// Save-file slot key for [`Inventory`].
pub const INVENTORY_SLOT: &str = "quest.inventory";

/// Stable IDs for every quest-relevant item.
pub mod item {
    pub const MIRROR_SHARD: &str = "mirror_shard";
}
