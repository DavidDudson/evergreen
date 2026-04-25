use bevy::prelude::*;
use serde::Deserialize;
use std::collections::HashMap;

/// Persistent key/value flag store used by dialogue choices.
///
/// Flags drive conditional branches: a choice option is only shown when its
/// [`Condition`] (or legacy `flags_required` AND-list) is satisfied.
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

/// Composable predicate over [`DialogueFlags`]. Lets choice options express
/// AND/OR/NOT logic in RON instead of just an AND-list of required flags.
///
/// Equivalent to `Condition::AllSet(flags_required)` for legacy entries.
#[derive(Debug, Clone, Deserialize)]
pub enum Condition {
    /// All flags must be set.
    AllSet(Vec<String>),
    /// At least one flag must be set.
    AnySet(Vec<String>),
    /// None of the flags may be set.
    NoneSet(Vec<String>),
    /// Logical AND of nested conditions.
    All(Vec<Condition>),
    /// Logical OR of nested conditions.
    Any(Vec<Condition>),
    /// Logical NOT of a nested condition.
    Not(Box<Condition>),
    /// Always-true.
    Always,
}

impl Condition {
    /// `serde(default)` helper -- equivalent to `Condition::Always`.
    pub fn always() -> Self {
        Self::Always
    }

    pub fn is_satisfied(&self, flags: &DialogueFlags) -> bool {
        match self {
            Self::AllSet(keys) => flags.all_set(keys),
            Self::AnySet(keys) => keys.iter().any(|k| flags.is_set(k)),
            Self::NoneSet(keys) => keys.iter().all(|k| !flags.is_set(k)),
            Self::All(conds) => conds.iter().all(|c| c.is_satisfied(flags)),
            Self::Any(conds) => conds.iter().any(|c| c.is_satisfied(flags)),
            Self::Not(cond) => !cond.is_satisfied(flags),
            Self::Always => true,
        }
    }
}
