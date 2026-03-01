use bevy::prelude::*;
use bevy::window::*;

const SCREEN_WIDTH_PX: u32 = 1280;
const SCREEN_HEIGHT_PX: u32 = 720;

pub fn window_plugin() -> WindowPlugin {
    WindowPlugin {
        primary_window: Some(Window {
            title: String::from("Evergreen: Summer of the Seventh Sunderance"),
            name: Some(String::from("evergreen.app")),
            resolution: WindowResolution::from((SCREEN_WIDTH_PX, SCREEN_HEIGHT_PX)),
            present_mode: PresentMode::AutoNoVsync,
            fit_canvas_to_parent: false,
            prevent_default_event_handling: true,
            window_theme: Some(WindowTheme::Dark),
            canvas: Some(String::from("#game-canvas")),
            ..default()
        }),
        ..default()
    }
}
