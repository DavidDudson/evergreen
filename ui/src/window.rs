use bevy::prelude::*;
use bevy::window::*;

const SCREEN_WIDTH_PX: u32 = 640;
const SCREEN_HEIGHT_PX: u32 = 360;

pub fn window_plugin() -> WindowPlugin {
    WindowPlugin {
        primary_window: Some(Window {
            title: String::from("Evergreen: Summer of the Seventh Sunderance"),
            name: Some(String::from("evergreen.app")),
            resolution: WindowResolution::from((SCREEN_WIDTH_PX, SCREEN_HEIGHT_PX)),
            present_mode: PresentMode::AutoNoVsync,
            fit_canvas_to_parent: true,
            prevent_default_event_handling: false,
            window_theme: Some(WindowTheme::Dark),
            ..default()
        }),
        ..default()
    }
}
