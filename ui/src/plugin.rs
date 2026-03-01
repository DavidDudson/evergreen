use bevy::prelude::*;
use models::game_states::GameState;

use crate::despawn::despawn_all;
use crate::focus;
use crate::game_over_menu::{self, GameOverMenu};
use crate::hud::{self, Hud};
use crate::main_menu::{self, MainMenu};
use crate::minimap;
use crate::pause_menu::{self, PauseMenu};

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::MainMenu), main_menu::setup)
            .add_systems(OnExit(GameState::MainMenu), despawn_all::<MainMenu>)
            .add_systems(Update, main_menu::button_system)
            .add_systems(OnEnter(GameState::Playing), (hud::setup, minimap::setup))
            .add_systems(
                OnExit(GameState::Playing),
                (despawn_all::<Hud>, minimap::despawn).run_if(not(in_state(GameState::Paused))),
            )
            .add_systems(
                Update,
                (hud::sync_petals, minimap::refresh).run_if(in_state(GameState::Playing)),
            )
            .add_systems(OnEnter(GameState::GameOver), game_over_menu::setup)
            .add_systems(OnExit(GameState::GameOver), despawn_all::<GameOverMenu>)
            .add_systems(Update, focus::handle_pause_input)
            .add_systems(OnEnter(GameState::Paused), pause_menu::setup)
            .add_systems(OnExit(GameState::Paused), despawn_all::<PauseMenu>);
    }
}
