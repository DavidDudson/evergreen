use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::asset::LoreCategory;

/// Persistent record of lore the player has discovered through dialogue.
///
/// Only scripts with `lore` metadata are recorded here. Casual conversation
/// without lore metadata is not stored.
#[derive(Resource, Debug, Default, Serialize, Deserialize)]
pub struct LoreBook {
    pub entries: Vec<LoreEntry>,
}

impl LoreBook {
    /// Returns `true` if a script with this id has already been recorded.
    pub fn has_seen(&self, script_id: &str) -> bool {
        self.entries.iter().any(|e| e.script_id == script_id)
    }

    /// Adds or merges an entry. If the script has been seen before,
    /// newly encountered `lines_seen` keys are appended (for branching scripts).
    pub fn record(&mut self, entry: LoreEntry) {
        if let Some(existing) = self
            .entries
            .iter_mut()
            .find(|e| e.script_id == entry.script_id)
        {
            for line in entry.lines_seen {
                if !existing.lines_seen.contains(&line) {
                    existing.lines_seen.push(line);
                }
            }
        } else {
            self.entries.push(entry);
        }
    }

    /// All unique categories that have at least one entry.
    pub fn categories(&self) -> Vec<LoreCategory> {
        let mut cats: Vec<LoreCategory> = self.entries.iter().map(|e| e.category).collect();
        cats.sort();
        cats.dedup();
        cats
    }

    /// All unique topics within a category.
    pub fn topics_in(&self, category: LoreCategory) -> Vec<String> {
        let mut topics: Vec<String> = self
            .entries
            .iter()
            .filter(|e| e.category == category)
            .map(|e| e.topic.clone())
            .collect();
        topics.sort();
        topics.dedup();
        topics
    }

    /// All entries for a given topic.
    pub fn entries_for_topic(&self, topic: &str) -> Vec<&LoreEntry> {
        self.entries.iter().filter(|e| e.topic == topic).collect()
    }

    /// Get the image path for a topic (from the first entry that has one).
    pub fn topic_image(&self, topic: &str) -> Option<&str> {
        self.entries
            .iter()
            .filter(|e| e.topic == topic)
            .find_map(|e| e.image.as_deref())
    }
}

/// A single lore entry representing a script the player has witnessed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoreEntry {
    /// Stable script identifier from the asset.
    pub script_id: String,
    /// Locale key for the speaker name.
    pub speaker_key: String,
    /// Tags for filtering in the Lore page.
    pub keyword_tags: Vec<String>,
    /// Lore category (Character, Place, Event, etc.).
    pub category: LoreCategory,
    /// Locale key for the topic name within the category.
    pub topic: String,
    /// Optional asset path to an image for this topic.
    pub image: Option<String>,
    /// Locale keys of all lines heard so far (may grow across multiple encounters).
    pub lines_seen: Vec<String>,
    /// Game time (seconds) when this entry was first created.
    pub game_time: f32,
}
