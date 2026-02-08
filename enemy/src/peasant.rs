use crate::enemy::{enemy_collision_layers, Enemy};
use avian2d::prelude::{
    AngularDamping, Collider, CollisionEventsEnabled, LinearDamping, LockedAxes,
};
use bevy::color::palettes::css::WHITE;
use bevy::prelude::*;
use models::attack::Attack;
use models::draggable::Draggable;
use models::hardness::Hardness;
use models::health::Health;
use models::name::Name;
use models::speed::Speed;
use models::textured::Textured;

#[derive(Component)]
#[require(Enemy, Name, Textured, Health, Speed, Attack)]
pub struct Peasant;

impl Peasant {
    pub fn spawn(
        commands: &mut Commands,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<ColorMaterial>>,
    ) {
        info!("Spawning peasant");
        commands
            .spawn((
                Draggable,
                Peasant,
                Name("Peasant".to_string()),
                Textured {
                    file: "enemy/Enemy.png".to_string(),
                },
                Health(5),
                Speed(100.),
                Attack::melee(Health(1)),
                Mesh2d(meshes.add(Rectangle::new(64., 64.))),
                MeshMaterial2d(materials.add(Color::from(WHITE))),
                Transform::from_xyz(-1920. / 2., 0., 0.),
                Collider::rectangle(64., 64.),
                LockedAxes::ROTATION_LOCKED,
                enemy_collision_layers(),
                Hardness(1),
            ))
            .insert((
                LinearDamping(0.5),
                AngularDamping(1.0),
                CollisionEventsEnabled,
            ));
    }
}
