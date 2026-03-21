use bevy::prelude::*;

use crate::remap::{CancelRemap, RemapCompleted, RequestRemap};
use crate::storage;
use crate::systems::{capture_remap_key, handle_cancel_remap, handle_request_remap};

pub struct KeybindsPlugin;

impl Plugin for KeybindsPlugin {
    fn build(&self, app: &mut App) {
        // Load saved bindings (or use defaults if nothing saved yet).
        let keybinds = storage::load().unwrap_or_default();
        app.insert_resource(keybinds);

        // Messages
        app.add_message::<RequestRemap>()
            .add_message::<CancelRemap>()
            .add_message::<RemapCompleted>();

        // Systems
        app.add_systems(
            Update,
            (handle_request_remap, handle_cancel_remap, capture_remap_key),
        );

        // Persist to storage whenever bindings change.
        app.add_systems(PostUpdate, storage::save_on_change);
    }
}
