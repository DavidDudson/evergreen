use bevy::prelude::States;

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
