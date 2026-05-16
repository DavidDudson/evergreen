//! Static quest data: definitions loaded from `.quest.ron` assets.

use models::alignment::AlignmentFaction;
use serde::{Deserialize, Serialize};

/// Stable identifier for a quest. Matches the `id` field in the asset and the
/// flag namespace `quest:<id>:...`.
#[derive(
    Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Deserialize, Serialize,
)]
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
