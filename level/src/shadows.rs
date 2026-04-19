//! Shared `Mesh2d` ellipse + `ColorMaterial` handle resource for drop shadows,
//! plus a single `spawn_drop_shadow` helper used by all asset spawn sites.

use bevy::math::Vec2;
use bevy::prelude::*;
use bevy::sprite_render::{ColorMaterial, MeshMaterial2d};
use models::palette::DROP_SHADOW;

/// Z-offset placing shadow just under its parent sprite (same layer).
const SHADOW_Z_OFFSET: f32 = -0.1;

/// Shared shadow assets. Spawned at `Startup` once, reused by every shadow.
#[derive(Resource)]
pub struct DropShadowAssets {
    pub mesh: Handle<Mesh>,
    pub material: Handle<ColorMaterial>,
}

/// Startup system: build the shared ellipse mesh + dark material.
pub fn init_shadow_assets(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let mesh = meshes.add(Circle::new(1.0));
    let material = materials.add(ColorMaterial::from(DROP_SHADOW));
    commands.insert_resource(DropShadowAssets { mesh, material });
}

/// Spawn one drop shadow as a child of `parent`. The shared circle mesh is
/// scaled by `half_size` to form an ellipse, offset down by `ground_offset_y`.
pub fn spawn_drop_shadow(
    commands: &mut Commands,
    assets: &DropShadowAssets,
    parent: Entity,
    half_size: Vec2,
    ground_offset_y: f32,
) {
    commands.spawn((
        Mesh2d(assets.mesh.clone()),
        MeshMaterial2d(assets.material.clone()),
        Transform::from_translation(Vec3::new(0.0, ground_offset_y, SHADOW_Z_OFFSET))
            .with_scale(half_size.extend(1.0)),
        ChildOf(parent),
    ));
}
