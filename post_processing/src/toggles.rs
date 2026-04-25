use bevy::prelude::*;

/// Marker component: skip bloom application for the camera that owns this.
///
/// Spawn alongside `Camera2d` to opt the camera out of bloom -- useful for
/// debug views or photo mode without tearing down the rest of the post chain.
#[derive(Component, Debug, Default, Clone, Copy)]
pub struct DisableBloom;

/// Marker component: skip color grading sync for cameras with this marker.
///
/// `sync_color_grading` filters cameras out via `Without<DisableColorGrading>`,
/// so the camera's `ColorGrading` is left as-is by the biome-driven system.
#[derive(Component, Debug, Default, Clone, Copy)]
pub struct DisableColorGrading;

/// Marker component: skip atmosphere darkness sync for entities with this marker.
///
/// `sync_atmosphere` filters out `BiomeAtmosphere` carriers via
/// `Without<DisableAtmosphere>`, leaving their darkness static.
#[derive(Component, Debug, Default, Clone, Copy)]
pub struct DisableAtmosphere;
