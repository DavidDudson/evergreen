use dialog::asset::LoreCategory;
use dialog::history::{LoreBook, LoreEntry};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct StoredLoreEntry {
    pub script_id: String,
    pub speaker_key: String,
    pub keyword_tags: Vec<String>,
    pub category: LoreCategory,
    pub topic: String,
    #[serde(default)]
    pub image: Option<String>,
    pub lines_seen: Vec<String>,
    pub game_time: f32,
}

impl From<&LoreEntry> for StoredLoreEntry {
    fn from(e: &LoreEntry) -> Self {
        Self {
            script_id: e.script_id.clone(),
            speaker_key: e.speaker_key.clone(),
            keyword_tags: e.keyword_tags.clone(),
            category: e.category,
            topic: e.topic.clone(),
            image: e.image.clone(),
            lines_seen: e.lines_seen.clone(),
            game_time: e.game_time,
        }
    }
}

impl From<StoredLoreEntry> for LoreEntry {
    fn from(s: StoredLoreEntry) -> Self {
        Self {
            script_id: s.script_id,
            speaker_key: s.speaker_key,
            keyword_tags: s.keyword_tags,
            category: s.category,
            topic: s.topic,
            image: s.image,
            lines_seen: s.lines_seen,
            game_time: s.game_time,
        }
    }
}

pub(crate) fn lore_book_to_stored(book: &LoreBook) -> Vec<StoredLoreEntry> {
    book.entries.iter().map(StoredLoreEntry::from).collect()
}

pub(crate) fn stored_to_lore_book(entries: Vec<StoredLoreEntry>) -> LoreBook {
    LoreBook {
        entries: entries.into_iter().map(LoreEntry::from).collect(),
    }
}
