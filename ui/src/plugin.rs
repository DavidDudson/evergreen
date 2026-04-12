use bevy::prelude::*;
use models::game_states::{GameState, should_despawn_world};

use crate::credits::{self, CreditsScreen};
use crate::despawn::despawn_all;
use crate::fonts;
use crate::dialog_box::{self, DialogBox};
use crate::focus;
use crate::game_over_menu::{self, GameOverMenu};
use crate::hud::{self, AlignmentBars, Hud};
use crate::keybind_screen::{self, KeybindScreen};
use crate::lore_page;
use crate::main_menu::{self, MainMenu};
use crate::minimap;
use crate::pause_menu::{self, PauseMenu};
use crate::settings_screen::{self, SettingsOrigin, SettingsScreen};

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<fonts::UiFont>();

        app.init_resource::<SettingsOrigin>();

        // Main menu
        app.add_systems(OnEnter(GameState::MainMenu), main_menu::setup)
            .add_systems(OnExit(GameState::MainMenu), despawn_all::<MainMenu>)
            .add_systems(Update, main_menu::button_system);

        // Playing HUD & minimap
        app.add_systems(
                OnEnter(GameState::Playing),
                (hud::setup, hud::setup_alignment_bars, minimap::setup),
            )
            .add_systems(
                OnExit(GameState::Playing),
                (despawn_all::<Hud>, despawn_all::<AlignmentBars>, minimap::despawn)
                    .run_if(should_despawn_world),
            )
            .add_systems(
                Update,
                (hud::sync_petals, hud::sync_alignment_bars, minimap::refresh)
                    .run_if(in_state(GameState::Playing)),
            );

        // Game over
        app.add_systems(OnEnter(GameState::GameOver), game_over_menu::setup)
            .add_systems(OnExit(GameState::GameOver), despawn_all::<GameOverMenu>);

        // Focus / pause
        app.add_systems(Update, focus::handle_pause_input)
            .add_systems(OnEnter(GameState::Paused), pause_menu::setup)
            .add_systems(OnExit(GameState::Paused), despawn_all::<PauseMenu>)
            .add_systems(
                Update,
                (pause_menu::handle_resume, pause_menu::handle_settings_button)
                    .run_if(in_state(GameState::Paused)),
            );

        // Dialog box (shown during NPC conversation)
        app.add_systems(OnEnter(GameState::Dialogue), dialog_box::setup)
            .add_systems(OnExit(GameState::Dialogue), despawn_all::<DialogBox>)
            .add_systems(
                Update,
                (
                    dialog_box::on_line_ready,
                    dialog_box::on_choices_ready,
                    dialog_box::handle_choice_interaction,
                )
                    .run_if(in_state(GameState::Dialogue)),
            );

        // Lore page
        app.add_systems(OnEnter(GameState::LorePage), lore_page::setup)
            .add_systems(OnExit(GameState::LorePage), lore_page::teardown)
            .add_systems(
                Update,
                (
                    lore_page::handle_back_button,
                    lore_page::handle_category_buttons,
                    lore_page::handle_topic_buttons,
                )
                    .run_if(in_state(GameState::LorePage)),
            );

        // Settings screen
        app.add_systems(OnEnter(GameState::Settings), settings_screen::setup)
            .add_systems(OnExit(GameState::Settings), despawn_all::<SettingsScreen>)
            .add_systems(
                Update,
                (
                    settings_screen::handle_volume_buttons,
                    settings_screen::handle_fullscreen_button,
                    settings_screen::handle_lang_buttons,
                    settings_screen::handle_keybinds_nav,
                    settings_screen::handle_reset,
                    settings_screen::handle_back,
                    settings_screen::sync_displays,
                )
                    .run_if(in_state(GameState::Settings)),
            );

        // Apply fullscreen whenever settings change (any state)
        app.add_systems(Update, settings_screen::apply_fullscreen);

        // Credits screen
        app.add_systems(OnEnter(GameState::Credits), credits::setup)
            .add_systems(OnExit(GameState::Credits), despawn_all::<CreditsScreen>)
            .add_systems(
                Update,
                (credits::handle_back, credits::sync_scrollbar)
                    .run_if(in_state(GameState::Credits)),
            );

        // Keybind config screen
        app.add_systems(OnEnter(GameState::KeybindConfig), keybind_screen::setup)
            .add_systems(OnExit(GameState::KeybindConfig), despawn_all::<KeybindScreen>)
            .add_systems(
                Update,
                (
                    keybind_screen::handle_key_buttons,
                    keybind_screen::handle_reset_buttons,
                    keybind_screen::handle_reset_all,
                    keybind_screen::handle_back,
                    keybind_screen::refresh_key_labels,
                    keybind_screen::sync_all_labels,
                    keybind_screen::sync_remap_overlay,
                )
                    .run_if(in_state(GameState::KeybindConfig)),
            );
    }
}
