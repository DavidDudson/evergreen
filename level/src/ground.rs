use avian2d::prelude::{Collider, CollisionEventsEnabled};
use bevy::color::palettes::css::GREEN;
use bevy::prelude::*;
use models::hardness::Hardness;
use models::name::Name;
use models::scenery::Scenery;

#[derive(Component, Default)]
#[require(Name, Scenery, Hardness)]
pub struct Ground;

impl Ground {
    pub fn spawn(
        commands: &mut Commands,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<ColorMaterial>>,
    ) {
        commands.spawn((
            Ground,
            Name("Ground".to_string()),
            Hardness(3),
            Mesh2d(meshes.add(Rectangle::new(5000., 1000.))),
            MeshMaterial2d(materials.add(Color::from(GREEN))),
            Transform::from_xyz(0., -480., 0.),
            Collider::rectangle(5000., 980.),
            CollisionEventsEnabled,
        ));
    }
}
