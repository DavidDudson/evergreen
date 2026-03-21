use bevy::prelude::*;
use models::game_states::GameState;

use crate::despawn::despawn_all;
use crate::dialog_box::{self, DialogBox};
use crate::focus;
use crate::game_over_menu::{self, GameOverMenu};
use crate::hud::{self, Hud};
use crate::lore_page;
use crate::main_menu::{self, MainMenu};
use crate::minimap;
use crate::pause_menu::{self, PauseMenu};

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        // Main menu
        app.add_systems(OnEnter(GameState::MainMenu), main_menu::setup)
            .add_systems(OnExit(GameState::MainMenu), despawn_all::<MainMenu>)
            .add_systems(Update, main_menu::button_system);

        // Playing HUD & minimap
        app.add_systems(OnEnter(GameState::Playing), (hud::setup, minimap::setup))
            .add_systems(
                OnExit(GameState::Playing),
                (despawn_all::<Hud>, minimap::despawn)
                    .run_if(not(in_state(GameState::Paused))),
            )
            .add_systems(
                Update,
                (hud::sync_petals, minimap::refresh).run_if(in_state(GameState::Playing)),
            );

        // Game over
        app.add_systems(OnEnter(GameState::GameOver), game_over_menu::setup)
            .add_systems(OnExit(GameState::GameOver), despawn_all::<GameOverMenu>);

        // Focus / pause
        app.add_systems(Update, focus::handle_pause_input)
            .add_systems(OnEnter(GameState::Paused), pause_menu::setup)
            .add_systems(OnExit(GameState::Paused), despawn_all::<PauseMenu>);

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
                (lore_page::handle_back_button, lore_page::handle_filter_buttons)
                    .run_if(in_state(GameState::LorePage)),
            );
    }
}
