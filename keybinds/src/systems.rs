use bevy::prelude::*;

use crate::bindings::Keybinds;
use crate::controller::ControllerBinds;
use crate::controller_serialize::button_name;
use crate::remap::{
    AwaitingControllerRemap, AwaitingRemap, CancelRemap, ControllerRemapCompleted, RemapCompleted,
    RequestControllerRemap, RequestRemap,
};

/// Handles [`RequestRemap`]: begins listening for the next keypress.
pub fn handle_request_remap(mut events: MessageReader<RequestRemap>, mut commands: Commands) {
    let Some(event) = events.read().next() else {
        return;
    };
    commands.insert_resource(AwaitingRemap {
        action: event.action,
    });
}

/// Handles [`CancelRemap`]: removes both awaiting-remap resources.
pub fn handle_cancel_remap(mut events: MessageReader<CancelRemap>, mut commands: Commands) {
    if events.read().next().is_some() {
        commands.remove_resource::<AwaitingRemap>();
        commands.remove_resource::<AwaitingControllerRemap>();
    }
}

/// Handles [`RequestControllerRemap`]: begins listening for the next button
/// press. Cancels any concurrent keyboard remap to avoid both firing.
pub fn handle_request_controller_remap(
    mut events: MessageReader<RequestControllerRemap>,
    mut commands: Commands,
) {
    let Some(event) = events.read().next() else {
        return;
    };
    commands.remove_resource::<AwaitingRemap>();
    commands.insert_resource(AwaitingControllerRemap {
        action: event.action,
    });
}

/// When `AwaitingControllerRemap` is active, captures the next gamepad button
/// press and binds it. Escape on the keyboard cancels without rebinding so
/// the user is never trapped if their gamepad disconnects mid-remap.
pub fn capture_remap_button(
    keyboard: Res<ButtonInput<KeyCode>>,
    awaiting: Option<Res<AwaitingControllerRemap>>,
    gamepads: Query<&Gamepad>,
    mut binds: ResMut<ControllerBinds>,
    mut commands: Commands,
    mut writer: MessageWriter<ControllerRemapCompleted>,
) {
    let Some(awaiting) = awaiting else { return };
    if keyboard.just_pressed(KeyCode::Escape) {
        commands.remove_resource::<AwaitingControllerRemap>();
        return;
    }
    let pressed = gamepads
        .iter()
        .find_map(|gp| gp.get_just_pressed().next().copied());
    let Some(button) = pressed else {
        return;
    };
    if button_name(button).is_none() {
        // Not a button we support persisting; ignore so the user can try again.
        return;
    }
    let action = awaiting.action;
    let had_conflict = binds.conflicts(action, button);
    binds.set(action, button);
    commands.remove_resource::<AwaitingControllerRemap>();
    writer.write(ControllerRemapCompleted {
        action,
        button,
        had_conflict,
    });
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
    writer.write(RemapCompleted {
        action,
        key: pressed,
        had_conflict,
    });
}
