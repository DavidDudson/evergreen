use bevy::prelude::*;

use crate::action::Action;

/// Inserted as a resource while the user is actively pressing a new key
/// for a specific action. The next keypress (other than Escape) binds it.
#[derive(Resource, Debug)]
pub struct AwaitingRemap {
    pub action: Action,
}

/// Message: request to open the remap UI for a specific action.
#[derive(bevy::prelude::Message, Debug, Clone)]
pub struct RequestRemap {
    pub action: Action,
}

/// Message: cancel any pending remap without changing bindings.
#[derive(bevy::prelude::Message, Debug, Clone)]
pub struct CancelRemap;

/// Message: emitted after a successful remap so the UI can refresh.
#[derive(bevy::prelude::Message, Debug, Clone)]
pub struct RemapCompleted {
    pub action: Action,
    pub key: KeyCode,
    pub had_conflict: bool,
}
