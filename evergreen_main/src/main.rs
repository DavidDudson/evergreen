use bevy::asset::AssetMetaCheck;
use bevy::diagnostic::LogDiagnosticsPlugin;
use bevy::prelude::*;
use camera::plugin::CameraPlugin;
use combat::plugin::CombatPlugin;
use level::plugin::LevelPlugin;
use models::game_states::GameState;
use player::plugin::PlayerPlugin;
use ui::plugin::UiPlugin;
use ui::window::window_plugin;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins
                .set(window_plugin())
                .set(ImagePlugin::default_nearest())
                .set(AssetPlugin {
                    meta_check: AssetMetaCheck::Never,
                    ..default()
                }),
            CameraPlugin,
            LogDiagnosticsPlugin::default(),
            UiPlugin,
            CombatPlugin,
            LevelPlugin,
            PlayerPlugin,
        ))
        .init_state::<GameState>()
        .run();
}
