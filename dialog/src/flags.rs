use bevy::prelude::*;
use std::collections::HashMap;

/// Persistent key/value flag store used by dialogue choices.
///
/// Flags drive conditional branches: a choice option is only shown when all
/// of its `flags_required` entries are present and `true` in this map.
/// Selecting a choice sets all entries in its `flags_set` list to `true`.
#[derive(Resource, Debug, Default, Clone)]
pub struct DialogueFlags(pub HashMap<String, bool>);

impl DialogueFlags {
    pub fn set(&mut self, key: impl Into<String>) {
        self.0.insert(key.into(), true);
    }

    pub fn is_set(&self, key: &str) -> bool {
        self.0.get(key).copied().unwrap_or(false)
    }

    pub fn all_set(&self, keys: &[String]) -> bool {
        keys.iter().all(|k| self.is_set(k))
    }
}
