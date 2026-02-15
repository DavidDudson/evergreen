use crate::enemy::{Enemy, ENEMY_COLLISION_GROUP};
use bevy::color::palettes::css::WHITE;
use bevy::prelude::*;
use bevy_rapier2d::prelude::{ActiveEvents, Collider, Damping, LockedAxes};
use models::attack::Attack;
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
                Collider::cuboid(64. / 2., 64. / 2.),
                LockedAxes::ROTATION_LOCKED,
                ENEMY_COLLISION_GROUP,
                ActiveEvents::CONTACT_FORCE_EVENTS,
                Hardness(1),
            ))
            .insert(Damping {
                linear_damping: 0.5,
                angular_damping: 1.0,
            });
    }
}
