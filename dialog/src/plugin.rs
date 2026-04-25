use bevy::prelude::*;
use models::alignment::PlayerAlignment;
use models::game_states::GameState;
use models::settings::GameSettings;

use crate::asset::{DialogueScript, DialogueScriptLoader};
use crate::barks::{tick_barks, BarkSelector};
use crate::events::{
    BarkFired, ChoiceMade, ChoicesReady, DialogueEnded, DialogueLineReady, StartDialogue,
};
use crate::flags::DialogueFlags;
use crate::history::LoreBook;
use crate::locale::{
    apply_locale_keys, sync_fallback_locale, sync_language, sync_locale, ActiveLocale,
    FallbackLocale, LocaleAsset, LocaleAssetLoader, LocaleMap, DEFAULT_LOCALE_CODE,
};
use crate::runner::{
    advance_runner, detect_interact_input, detect_interact_range, handle_choice, on_dialogue_ended,
    start_dialogue, DialogueRunner, DialogueTarget,
};

pub struct DialogPlugin;

impl Plugin for DialogPlugin {
    fn build(&self, app: &mut App) {
        // Assets & loaders
        app.init_asset::<DialogueScript>()
            .init_asset::<LocaleAsset>()
            .init_asset_loader::<DialogueScriptLoader>()
            .init_asset_loader::<LocaleAssetLoader>();

        // Resources
        app.init_resource::<DialogueFlags>()
            .init_resource::<LoreBook>()
            .init_resource::<LocaleMap>()
            .init_resource::<DialogueRunner>()
            .init_resource::<DialogueTarget>()
            .init_resource::<PlayerAlignment>()
            .init_resource::<BarkSelector>();

        // Messages
        app.add_message::<StartDialogue>()
            .add_message::<DialogueLineReady>()
            .add_message::<ChoicesReady>()
            .add_message::<ChoiceMade>()
            .add_message::<DialogueEnded>()
            .add_message::<BarkFired>();

        // Startup: load the locale specified in GameSettings (set by SavePlugin).
        app.add_systems(Startup, load_initial_locale);

        // Locale sync and language switching (runs always). `apply_locale_keys`
        // must run after `sync_locale`/`sync_fallback_locale` so freshly-loaded
        // strings reach `LocaleKey`-tagged Text nodes the same frame.
        app.add_systems(
            Update,
            (
                sync_locale,
                sync_fallback_locale,
                sync_language,
                apply_locale_keys,
            )
                .chain(),
        );

        // Playing: range detection, interact input, barks, start_dialogue
        app.add_systems(
            Update,
            (
                detect_interact_range,
                detect_interact_input,
                tick_barks,
                start_dialogue,
            )
                .run_if(in_state(GameState::Playing)),
        );

        // Dialogue state: runner systems
        app.add_systems(
            Update,
            (advance_runner, handle_choice, on_dialogue_ended)
                .chain()
                .run_if(in_state(GameState::Dialogue)),
        );
    }
}

fn load_initial_locale(
    mut commands: Commands,
    mut locale_map: ResMut<LocaleMap>,
    asset_server: Res<AssetServer>,
    settings: Res<GameSettings>,
) {
    let active_path = format!("locale/{}.locale.ron", settings.language);
    commands.insert_resource(ActiveLocale(asset_server.load(active_path)));

    let fallback_path = format!("locale/{DEFAULT_LOCALE_CODE}.locale.ron");
    commands.insert_resource(FallbackLocale(asset_server.load(fallback_path)));

    locale_map.set_active_code(&settings.language);
}
