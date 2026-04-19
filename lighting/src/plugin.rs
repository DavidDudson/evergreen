use bevy::prelude::*;
use bevy_light_2d::plugin::Light2dPlugin;

/// Top-level lighting plugin -- composes `bevy_light_2d` + project systems.
pub struct LightingPlugin;

impl Plugin for LightingPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(Light2dPlugin);
    }
}
