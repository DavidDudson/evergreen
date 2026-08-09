//! Static quest data: definitions loaded from `.quest.ron` assets.

use models::alignment::AlignmentFaction;
use serde::{Deserialize, Serialize};

/// Stable identifier for a quest. Matches the `id` field in the asset and the
/// flag namespace `quest:<id>:...`.
///
/// `transparent` so it reads and writes as a bare string. Without it, RON
/// demands the newtype form -- `id: QuestId("bigby.sick_animals")` -- which
/// clashes with every sibling key in a `.quest.ron` being a plain string, and
/// which every new quest file would have to remember.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Deserialize, Serialize)]
#[serde(transparent)]
pub struct QuestId(pub String);

impl QuestId {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&str> for QuestId {
    fn from(value: &str) -> Self {
        Self(value.to_owned())
    }
}

/// One linear step inside a quest. Index in [`Quest::milestones`] is its
/// `MilestoneIndex`; the `flag` field is the dialogue flag whose set state
/// marks the milestone complete.
#[derive(Debug, Clone, Deserialize)]
pub struct Milestone {
    /// Locale key for the milestone's display label.
    pub label_key: String,
    /// `DialogueFlags` key whose presence completes this milestone.
    /// By convention `quest:<quest_id>:milestone:<index>`.
    pub flag: String,
}

/// Reward applied when the final milestone is reached.
#[derive(Debug, Clone, Deserialize)]
pub enum Unlock {
    /// Grant +1 to a faction alignment.
    Alignment(AlignmentFaction),
    /// Set a free-form dialogue flag.
    Flag(String),
}

/// Lore tied to the quest, surfaced in the quest log.
#[derive(Debug, Clone, Deserialize)]
pub struct QuestLore {
    /// Locale key for the lore body shown in the quest log content panel.
    pub body_key: String,
}

/// Loaded quest definition.
#[derive(Debug, Clone, Deserialize)]
pub struct Quest {
    pub id: QuestId,
    /// Locale key for the quest title.
    pub title_key: String,
    /// Locale key for the quest description / pitch.
    pub description_key: String,
    /// Locale key for the giver's display name (e.g. `npc.bigby.name`).
    pub giver_key: String,
    /// Flag set when the quest becomes available (e.g. on first interaction).
    /// By convention `quest:<id>:offer`.
    pub offer_flag: String,
    /// Flag set when the player accepts the quest (it joins the active log).
    /// By convention `quest:<id>:accept`.
    pub accept_flag: String,
    /// Linear milestone progression.
    pub milestones: Vec<Milestone>,
    /// Lore content shown in the quest log.
    pub lore: QuestLore,
    /// Rewards applied on quest completion.
    #[serde(default)]
    pub unlocks: Vec<Unlock>,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Directory holding the shipped `.quest.ron` assets.
    fn quests_dir() -> std::path::PathBuf {
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../assets/quests")
    }

    /// Every shipped quest asset must parse. This is the only place the
    /// authored RON format is checked at build time -- a malformed file
    /// otherwise fails silently at runtime as an `AssetServer` load error.
    #[test]
    fn all_shipped_quest_assets_parse() {
        let dir = quests_dir();
        let entries =
            std::fs::read_dir(&dir).unwrap_or_else(|e| panic!("read {}: {e}", dir.display()));

        let files: Vec<_> = entries
            .filter_map(Result::ok)
            .map(|e| e.path())
            .filter(|p| p.to_string_lossy().ends_with(".quest.ron"))
            .collect();

        assert!(
            !files.is_empty(),
            "no .quest.ron files in {}",
            dir.display()
        );

        for path in files {
            let text = std::fs::read_to_string(&path)
                .unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
            let quest: Quest =
                ron::from_str(&text).unwrap_or_else(|e| panic!("parse {}: {e}", path.display()));
            assert!(
                !quest.id.as_str().is_empty(),
                "{} has an empty id",
                path.display()
            );
        }
    }

    /// `QuestId` is authored as a bare string, not RON's newtype form.
    #[test]
    fn quest_id_parses_from_bare_string() {
        let id: QuestId = ron::from_str("\"bigby.sick_animals\"").expect("bare string parses");
        assert_eq!(id.as_str(), "bigby.sick_animals");
    }

    /// `#[serde(transparent)]` must not change the save format: `QuestId` is a
    /// JSON map key in `QuestProgress`, so it has to stay a plain string.
    #[test]
    fn quest_id_serializes_as_plain_json_string() {
        let json = serde_json::to_string(&QuestId::from("bigby.sick_animals")).expect("serializes");
        assert_eq!(json, "\"bigby.sick_animals\"");
    }
}
