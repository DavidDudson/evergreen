use bevy::diagnostic::{EntityCountDiagnosticsPlugin, FrameTimeDiagnosticsPlugin};
use bevy::prelude::*;

#[cfg(debug_assertions)]
use crate::overlay::{
    overlay_visible, setup_overlay, toggle_overlay, update_overlay, OverlayState,
};

pub struct DiagnosticsPlugin;

#[cfg(debug_assertions)]
impl Plugin for DiagnosticsPlugin {
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
}
