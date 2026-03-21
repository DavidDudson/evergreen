use bevy::prelude::*;

use crate::bindings::Keybinds;
use crate::remap::{AwaitingRemap, CancelRemap, RemapCompleted, RequestRemap};

/// Handles [`RequestRemap`]: begins listening for the next keypress.
pub fn handle_request_remap(
    mut events: MessageReader<RequestRemap>,
    mut commands: Commands,
) {
    let Some(event) = events.read().next() else {
        return;
    };
    commands.insert_resource(AwaitingRemap { action: event.action });
}

/// Handles [`CancelRemap`]: removes the awaiting state.
pub fn handle_cancel_remap(
    mut events: MessageReader<CancelRemap>,
    mut commands: Commands,
) {
    if events.read().next().is_some() {
        commands.remove_resource::<AwaitingRemap>();
    }
}

/// When `AwaitingRemap` is active, captures the next keypress and binds it.
/// Escape cancels without rebinding.
pub fn capture_remap_key(
    keyboard: Res<ButtonInput<KeyCode>>,
    awaiting: Option<Res<AwaitingRemap>>,
    mut keybinds: ResMut<Keybinds>,
    mut commands: Commands,
    mut writer: MessageWriter<RemapCompleted>,
) {
    let Some(awaiting) = awaiting else { return };

    // Any press cancels the remap wait.
    let Some(pressed) = keyboard.get_just_pressed().next().copied() else {
        return;
    };

    if pressed == KeyCode::Escape {
        commands.remove_resource::<AwaitingRemap>();
        return;
    }

    let action = awaiting.action;
    let had_conflict = keybinds.conflicts(action, pressed);
    keybinds.set(action, pressed);
    commands.remove_resource::<AwaitingRemap>();
    writer.write(RemapCompleted { action, key: pressed, had_conflict });
}
