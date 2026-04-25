use bevy::prelude::*;

use crate::credits::CreditsScreenSetup;
use crate::dialog_box::DialogBoxScreen;
use crate::focus;
use crate::fonts;
use crate::game_over_menu::GameOverScreen;
use crate::hud::HudScreen;
use crate::keybind_screen::KeybindScreenSetup;
use crate::level_complete::LevelCompleteSetup;
use crate::lore_page::LoreScreen;
use crate::main_menu::MainMenuScreen;
use crate::pause_menu::{PauseScreen, QuitToMenuRequested};
use crate::screen::ScreenSetup;
use crate::settings_screen::SettingsScreenSetup;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<fonts::UiFont>();
        app.init_resource::<QuitToMenuRequested>();

        // Cross-cutting input handler.
        app.add_systems(Update, focus::handle_pause_input);

        // Each screen owns its own state-transition wiring via ScreenSetup.
        // To add a new screen, implement ScreenSetup in its module and add a
        // call here -- no per-screen match in this plugin.
        MainMenuScreen::register(app);
        HudScreen::register(app);
        GameOverScreen::register(app);
        PauseScreen::register(app);
        DialogBoxScreen::register(app);
        LoreScreen::register(app);
        SettingsScreenSetup::register(app);
        CreditsScreenSetup::register(app);
        KeybindScreenSetup::register(app);
        LevelCompleteSetup::register(app);
    }
}
