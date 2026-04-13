use bevy::asset::{io::Reader, Asset, AssetLoader, LoadContext};
use bevy::reflect::TypePath;
use models::alignment::AlignmentFaction;
use serde::Deserialize;

/// Broad category for lore entries displayed in the lore page sidebar.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, serde::Serialize)]
pub enum LoreCategory {
    Character,
    Place,
    Event,
    Disease,
    Item,
    Faction,
}

impl LoreCategory {
    /// Locale key for the category display name.
    pub fn locale_key(self) -> &'static str {
        match self {
            Self::Character => "ui.lore.category.character",
            Self::Place => "ui.lore.category.place",
            Self::Event => "ui.lore.category.event",
            Self::Disease => "ui.lore.category.disease",
            Self::Item => "ui.lore.category.item",
            Self::Faction => "ui.lore.category.faction",
        }
    }
}

/// Metadata marking a dialogue script as lore-contributing.
/// Scripts without this field are casual conversation and not recorded.
#[derive(Debug, Clone, Deserialize)]
pub struct LoreMeta {
    /// Which sidebar category this lore belongs to.
    pub category: LoreCategory,
    /// Locale key for the topic name (e.g. `"lore.character.mordred"`).
    pub topic: String,
    /// Optional asset path to an image shown on the topic page (e.g. character portrait).
    #[serde(default)]
    pub image: Option<String>,
}

/// A dialogue script loaded from a `.dialog.ron` asset file.
///
/// All strings are locale keys resolved at render time by [`crate::locale::LocaleMap`].
#[derive(Asset, TypePath, Debug, Deserialize, Clone)]
pub struct DialogueScript {
    /// Unique stable identifier used to index lore entries.
    pub id: String,
    /// Locale key for the speaker's display name (e.g. `"npc.elder.name"`).
    pub speaker_key: String,
    /// Tags used to filter entries in the Lore page (e.g. `["history", "nature"]`).
    pub keyword_tags: Vec<String>,
    /// If present, this script contributes to the lore book when witnessed.
    /// Scripts without this field are not recorded.
    #[serde(default)]
    pub lore: Option<LoreMeta>,
    /// Ordered sequence of lines/choices.
    pub lines: Vec<DialogueLine>,
}

/// A single step in a dialogue script.
#[derive(Debug, Deserialize, Clone)]
pub enum DialogueLine {
    /// The speaker says something. `text_key` is a locale key.
    Speech { text_key: String },
    /// The player must choose from a set of options.
    PlayerChoice { options: Vec<ChoiceOption> },
}

/// One branch inside a [`DialogueLine::PlayerChoice`].
#[derive(Debug, Deserialize, Clone)]
pub struct ChoiceOption {
    /// Locale key for the option label shown to the player.
    pub text_key: String,
    /// Flags that must all be `true` for this option to be shown.
    /// Empty means always visible.
    pub flags_required: Vec<String>,
    /// Flags to set to `true` when this option is selected.
    pub flags_set: Vec<String>,
    /// If set, grant +1 to this alignment faction when the choice is made.
    #[serde(default)]
    pub alignment_grant: Option<AlignmentFaction>,
    /// Sub-lines to run after this choice is made.
    pub next: Vec<DialogueLine>,
}

// ---------------------------------------------------------------------------
// Asset loader
// ---------------------------------------------------------------------------

/// Loads `.dialog.ron` files into [`DialogueScript`] assets.
#[derive(Default, TypePath)]
pub struct DialogueScriptLoader;

impl AssetLoader for DialogueScriptLoader {
    type Asset = DialogueScript;
    type Settings = ();
    type Error = Box<dyn std::error::Error + Send + Sync + 'static>;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &(),
        _ctx: &mut LoadContext<'_>,
    ) -> Result<DialogueScript, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        let text = std::str::from_utf8(&bytes)?;
        Ok(ron::from_str(text)?)
    }

    fn extensions(&self) -> &[&str] {
        &["dialog.ron"]
    }
}
