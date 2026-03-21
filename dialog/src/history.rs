use bevy::prelude::*;

/// Persistent record of all dialogue the player has witnessed.
///
/// Only entries from scripts that have actually been presented to the player
/// are stored here — progressive disclosure is enforced by the runner.
#[derive(Resource, Debug, Default)]
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
    pub fn record(
        &mut self,
        script_id: impl Into<String>,
        speaker_key: impl Into<String>,
        keyword_tags: Vec<String>,
        lines_seen: Vec<String>,
        game_time: f32,
    ) {
        let id = script_id.into();
        if let Some(existing) = self.entries.iter_mut().find(|e| e.script_id == id) {
            for line in lines_seen {
                if !existing.lines_seen.contains(&line) {
                    existing.lines_seen.push(line);
                }
            }
        } else {
            self.entries.push(LoreEntry {
                script_id: id,
                speaker_key: speaker_key.into(),
                keyword_tags,
                lines_seen,
                game_time,
            });
        }
    }
}

/// A single lore entry representing a script the player has witnessed.
#[derive(Debug, Clone)]
pub struct LoreEntry {
    /// Stable script identifier from the asset.
    pub script_id: String,
    /// Locale key for the speaker name.
    pub speaker_key: String,
    /// Tags for filtering in the Lore page.
    pub keyword_tags: Vec<String>,
    /// Locale keys of all lines heard so far (may grow across multiple encounters).
    pub lines_seen: Vec<String>,
    /// Game time (seconds) when this entry was first created.
    pub game_time: f32,
}
