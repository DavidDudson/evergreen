use bevy::prelude::*;

#[derive(States, Debug, Clone, Copy, Eq, PartialEq, Hash, Default)]
pub enum GameState {
    #[default]
    MainMenu,
    Playing,
    Paused,
    GameOver,
    /// NPC conversation in progress; world systems are frozen.
    Dialogue,
    /// Lore browser accessed from the main menu.
    LorePage,
    /// Key remapping UI, accessible from the settings screen.
    KeybindConfig,
    /// Settings hub: audio, video, and keybind navigation.
    Settings,
    /// Credits screen, accessible from the main menu.
    Credits,
}

/// Run condition: true when leaving `Playing` for a state that should
/// tear down the world (i.e. NOT `Paused`, `Dialogue`, `KeybindConfig`,
/// or `Settings`).
pub fn should_despawn_world(state: Res<State<GameState>>) -> bool {
    !matches!(
        state.get(),
        GameState::Paused
            | GameState::Dialogue
            | GameState::KeybindConfig
            | GameState::Settings
    )
}
