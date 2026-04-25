use bevy::asset::AssetMetaCheck;
use bevy::prelude::*;
use camera::plugin::CameraPlugin;
use combat::plugin::CombatPlugin;
use diagnostics::plugin::DiagnosticsPlugin;
use dialog::DialogPlugin;
use keybinds::KeybindsPlugin;
use level::plugin::LevelPlugin;
use lighting::plugin::LightingPlugin;
use models::game_states::GameState;
use models::palette::PaletteTheme;
use player::plugin::PlayerPlugin;
use post_processing::plugin::PostProcessingPlugin;
use save::SavePlugin;
use ui::plugin::UiPlugin;
use ui::window::window_plugin;

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(window_plugin())
                .set(ImagePlugin::default_nearest())
                .set(AssetPlugin {
                    meta_check: AssetMetaCheck::Never,
                    ..default()
                }),
        )
        .add_plugins((
            CameraPlugin,
            DiagnosticsPlugin,
            KeybindsPlugin,
            DialogPlugin,
            SavePlugin,
            UiPlugin,
            CombatPlugin,
            LevelPlugin,
            PlayerPlugin,
            PostProcessingPlugin,
            LightingPlugin,
        ))
        .init_state::<GameState>()
        .init_resource::<PaletteTheme>()
        .run();
}
