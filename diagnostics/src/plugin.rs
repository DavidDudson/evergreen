#[cfg(debug_assertions)]
use bevy::diagnostic::{EntityCountDiagnosticsPlugin, FrameTimeDiagnosticsPlugin};
use bevy::prelude::*;

#[cfg(debug_assertions)]
use crate::debug_panel::{
    handle_debug_input, panel_visible, setup_debug_panel, toggle_debug_panel,
    update_debug_panel, DebugPanelState,
};
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
        app.init_resource::<DebugPanelState>();
        app.add_systems(Startup, (setup_overlay, setup_debug_panel));
        app.add_systems(Update, (toggle_overlay, toggle_debug_panel));
        app.add_systems(Update, update_overlay.run_if(overlay_visible));
        app.add_systems(
            Update,
            (handle_debug_input, update_debug_panel).run_if(panel_visible),
        );
    }

    #[cfg(not(debug_assertions))]
    fn build(&self, _app: &mut App) {}
}
