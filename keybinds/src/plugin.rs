use bevy::prelude::*;

use crate::bindings::Keybinds;
use crate::controller::ControllerBinds;
use crate::remap::{
    CancelRemap, ControllerRemapCompleted, RemapCompleted, RequestControllerRemap, RequestRemap,
};
use crate::systems::{
    capture_remap_button, capture_remap_key, handle_cancel_remap,
    handle_request_controller_remap, handle_request_remap,
};

pub struct KeybindsPlugin;

impl Plugin for KeybindsPlugin {
    fn build(&self, app: &mut App) {
        // Default bindings; SavePlugin overwrites these in PreStartup.
        app.insert_resource(Keybinds::default());
        app.insert_resource(ControllerBinds::default());

        // Messages
        app.add_message::<RequestRemap>()
            .add_message::<RequestControllerRemap>()
            .add_message::<CancelRemap>()
            .add_message::<RemapCompleted>()
            .add_message::<ControllerRemapCompleted>();

        // Systems
        app.add_systems(
            Update,
            (
                handle_request_remap,
                handle_request_controller_remap,
                handle_cancel_remap,
                capture_remap_key,
                capture_remap_button,
            ),
        );
    }
}
