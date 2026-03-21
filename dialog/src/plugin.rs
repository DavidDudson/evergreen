use bevy::prelude::*;
use models::game_states::GameState;
use models::settings::GameSettings;

use crate::asset::{DialogueScript, DialogueScriptLoader};
use crate::barks::tick_barks;
use crate::events::{
    BarkFired, ChoiceMade, ChoicesReady, DialogueEnded, DialogueLineReady, StartDialogue,
};
use crate::flags::DialogueFlags;
use crate::history::LoreBook;
use crate::locale::{
    ActiveLocale, LocaleAsset, LocaleAssetLoader, LocaleMap, sync_language, sync_locale,
};
use crate::runner::{
    DialogueRunner, advance_runner, detect_interact_input, detect_interact_range,
    handle_choice, on_dialogue_ended, start_dialogue,
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
            .init_resource::<DialogueRunner>();

        // Messages
        app.add_message::<StartDialogue>()
            .add_message::<DialogueLineReady>()
            .add_message::<ChoicesReady>()
            .add_message::<ChoiceMade>()
            .add_message::<DialogueEnded>()
            .add_message::<BarkFired>();

        // Startup: load the locale specified in GameSettings (set by SavePlugin).
        app.add_systems(Startup, load_initial_locale);

        // Locale sync and language switching (runs always)
        app.add_systems(Update, (sync_locale, sync_language));

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
    asset_server: Res<AssetServer>,
    settings: Res<GameSettings>,
) {
    let path = format!("locale/{}.locale.ron", settings.language);
    commands.insert_resource(ActiveLocale(asset_server.load(path)));
}
