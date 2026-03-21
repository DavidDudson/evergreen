use bevy::prelude::*;

use crate::bindings::Keybinds;
use crate::remap::{CancelRemap, RemapCompleted, RequestRemap};
use crate::systems::{capture_remap_key, handle_cancel_remap, handle_request_remap};

pub struct KeybindsPlugin;

impl Plugin for KeybindsPlugin {
    fn build(&self, app: &mut App) {
        // Default bindings; SavePlugin overwrites these in PreStartup.
        app.insert_resource(Keybinds::default());

        // Messages
        app.add_message::<RequestRemap>()
            .add_message::<CancelRemap>()
            .add_message::<RemapCompleted>();

        // Systems
        app.add_systems(
            Update,
            (handle_request_remap, handle_cancel_remap, capture_remap_key),
        );
    }
}
