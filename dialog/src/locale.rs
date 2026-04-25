use bevy::asset::{io::Reader, Asset, AssetId, AssetLoader, LoadContext};
use bevy::prelude::*;
use bevy::reflect::TypePath;
use models::settings::GameSettings;
use std::collections::HashMap;

/// All supported locales as `(code, display_name)` pairs.
/// Add entries here when adding new locale files under `assets/locale/`.
pub const AVAILABLE_LOCALES: &[(&str, &str)] = &[("en-US", "English"), ("es-ES", "Espanol")];

/// A flat key→string locale map loaded from a `.locale.ron` file.
#[derive(Asset, TypePath, Debug, Default, Clone)]
pub struct LocaleAsset(pub HashMap<String, String>);

/// Default fallback locale for missing keys (English).
pub const DEFAULT_LOCALE_CODE: &str = "en-US";

/// Runtime resource holding the currently active locale data + a fallback
/// chain. Lookup order:
///   1. exact-key in active locale
///   2. exact-key in default locale (e.g. en-US) -- if different
///   3. the key itself (debug fallthrough)
#[derive(Resource, Debug, Default)]
pub struct LocaleMap {
    data: HashMap<String, String>,
    fallback: HashMap<String, String>,
    active_code: String,
}

impl LocaleMap {
    /// Look up a locale key. Falls through active → default → key.
    pub fn get<'a>(&'a self, key: &'a str) -> &'a str {
        if let Some(value) = self.data.get(key) {
            return value.as_str();
        }
        if let Some(value) = self.fallback.get(key) {
            return value.as_str();
        }
        key
    }

    /// Replace the active locale data.
    pub fn load_from(&mut self, asset: &LocaleAsset) {
        self.data = asset.0.clone();
    }

    /// Replace the default-locale fallback table.
    pub fn load_fallback(&mut self, asset: &LocaleAsset) {
        self.fallback = asset.0.clone();
    }

    /// Track the currently active locale code.
    pub fn set_active_code(&mut self, code: impl Into<String>) {
        self.active_code = code.into();
    }

    /// Currently active locale code (e.g. `"es-ES"`).
    pub fn active_code(&self) -> &str {
        &self.active_code
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

/// Holds the default-locale asset handle (e.g. `en-US`) used as fallback
/// when the active locale is missing keys.
#[derive(Resource)]
pub struct FallbackLocale(pub Handle<LocaleAsset>);

/// System: copies the active locale asset into [`LocaleMap`] once loaded,
/// and re-copies whenever the active locale changes (language switching).
///
/// Tracks the last-loaded [`AssetId`] so it only re-runs after a language
/// change, not every frame.
pub fn sync_locale(
    active: Res<ActiveLocale>,
    assets: Res<Assets<LocaleAsset>>,
    mut locale_map: ResMut<LocaleMap>,
    mut loaded_id: Local<Option<AssetId<LocaleAsset>>>,
) {
    let current_id = active.0.id();
    if *loaded_id == Some(current_id) {
        return;
    }
    if let Some(asset) = assets.get(current_id) {
        locale_map.load_from(asset);
        *loaded_id = Some(current_id);
    }
}

/// System: copies the fallback locale asset into [`LocaleMap::fallback`].
/// Runs once per fallback-handle change.
pub fn sync_fallback_locale(
    fallback: Res<FallbackLocale>,
    assets: Res<Assets<LocaleAsset>>,
    mut locale_map: ResMut<LocaleMap>,
    mut loaded_id: Local<Option<AssetId<LocaleAsset>>>,
) {
    let current_id = fallback.0.id();
    if *loaded_id == Some(current_id) {
        return;
    }
    if let Some(asset) = assets.get(current_id) {
        locale_map.load_fallback(asset);
        *loaded_id = Some(current_id);
    }
}

/// System: when [`GameSettings::language`] changes, reload the locale asset.
pub fn sync_language(
    settings: Res<GameSettings>,
    mut active: ResMut<ActiveLocale>,
    mut locale_map: ResMut<LocaleMap>,
    asset_server: Res<AssetServer>,
) {
    if !settings.is_changed() {
        return;
    }
    let path = format!("locale/{}.locale.ron", settings.language);
    *active = ActiveLocale(asset_server.load(path));
    locale_map.set_active_code(&settings.language);
}
