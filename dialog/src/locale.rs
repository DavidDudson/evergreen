use bevy::asset::{Asset, AssetLoader, LoadContext, io::Reader};
use bevy::prelude::*;
use bevy::reflect::TypePath;
use std::collections::HashMap;

/// A flat key→string locale map loaded from a `.locale.ron` file.
///
/// All keys follow dot notation: `"npc.merchant.greeting"`.
/// Missing keys fall back to the raw key so nothing panics in development.
///
/// This is intentionally simple. It can be replaced with `bevy_fluent` later
/// without changing callsites — just swap the lookup impl.
#[derive(Asset, TypePath, Debug, Default, Clone)]
pub struct LocaleAsset(pub HashMap<String, String>);

/// Runtime resource holding the currently active locale data.
#[derive(Resource, Debug, Default)]
pub struct LocaleMap {
    data: HashMap<String, String>,
}

impl LocaleMap {
    /// Look up a locale key. Returns the key itself if no translation exists.
    pub fn get<'a>(&'a self, key: &'a str) -> &'a str {
        self.data.get(key).map(String::as_str).unwrap_or(key)
    }

    /// Replace the active locale data.
    pub fn load_from(&mut self, asset: &LocaleAsset) {
        self.data = asset.0.clone();
    }
}

// ---------------------------------------------------------------------------
// Asset loader
// ---------------------------------------------------------------------------

/// Loads `.locale.ron` files into [`LocaleAsset`].
#[derive(Default, TypePath)]
pub struct LocaleAssetLoader;

impl AssetLoader for LocaleAssetLoader {
    type Asset = LocaleAsset;
    type Settings = ();
    type Error = Box<dyn std::error::Error + Send + Sync + 'static>;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &(),
        _ctx: &mut LoadContext<'_>,
    ) -> Result<LocaleAsset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        let text = std::str::from_utf8(&bytes)?;
        let map: HashMap<String, String> = ron::from_str(text)?;
        Ok(LocaleAsset(map))
    }

    fn extensions(&self) -> &[&str] {
        &["locale.ron"]
    }
}

// ---------------------------------------------------------------------------
// Handle resource
// ---------------------------------------------------------------------------

/// Holds the active locale asset handle so it isn't dropped.
#[derive(Resource)]
pub struct ActiveLocale(pub Handle<LocaleAsset>);

/// System: once the active locale asset has loaded, update [`LocaleMap`].
///
/// Uses a `Local<bool>` flag so the copy only happens once per locale load.
/// If you add hot-reload support later, remove the guard.
pub fn sync_locale(
    active: Res<ActiveLocale>,
    assets: Res<Assets<LocaleAsset>>,
    mut locale_map: ResMut<LocaleMap>,
    mut done: Local<bool>,
) {
    if *done {
        return;
    }
    if let Some(asset) = assets.get(active.0.id()) {
        locale_map.load_from(asset);
        *done = true;
    }
}
