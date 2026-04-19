#[cfg(debug_assertions)]
use bevy::diagnostic::{EntityCountDiagnosticsPlugin, FrameTimeDiagnosticsPlugin};
use bevy::prelude::*;

#[cfg(debug_assertions)]
use crate::overlay::{
    overlay_visible, setup_overlay, toggle_overlay, update_overlay, OverlayState,
};

pub struct DiagnosticsPlugin;

impl Plugin for DiagnosticsPlugin {
    #[cfg(debug_assertions)]
    fn build(&self, app: &mut App) {
        app.add_plugins((
            FrameTimeDiagnosticsPlugin::default(),
            EntityCountDiagnosticsPlugin::default(),
        ));

        app.init_resource::<OverlayState>();
        app.add_systems(Startup, setup_overlay);
        app.add_systems(Update, toggle_overlay);
        app.add_systems(Update, update_overlay.run_if(overlay_visible));
    }

    #[cfg(not(debug_assertions))]
    fn build(&self, _app: &mut App) {}
}
