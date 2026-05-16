//! Asset loader for `.quest.ron` files.

use bevy::asset::{io::Reader, Asset, AssetLoader, LoadContext};
use bevy::reflect::TypePath;

use crate::model::Quest;

/// Wraps a [`Quest`] for the asset system.
#[derive(Asset, TypePath, Debug, Clone)]
pub struct QuestAsset(pub Quest);

#[derive(Default, TypePath)]
pub struct QuestAssetLoader;

impl AssetLoader for QuestAssetLoader {
    type Asset = QuestAsset;
    type Settings = ();
    type Error = Box<dyn std::error::Error + Send + Sync + 'static>;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &(),
        _ctx: &mut LoadContext<'_>,
    ) -> Result<QuestAsset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        let text = std::str::from_utf8(&bytes)?;
        let quest: Quest = ron::from_str(text)?;
        Ok(QuestAsset(quest))
    }

    fn extensions(&self) -> &[&str] {
        &["quest.ron"]
    }
}
