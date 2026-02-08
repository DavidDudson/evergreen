use avian2d::prelude::{CollisionLayers, LayerMask, LinearVelocity, PhysicsLayer, RigidBody};
use bevy::prelude::Component;
use models::attack::Attack;
use models::hardness::Hardness;
use models::health::Health;
use models::name::Name;
use models::speed::Speed;
use models::textured::Textured;

// Define collision layers for enemies
#[derive(PhysicsLayer, Clone, Copy, Debug, Default)]
pub enum GameLayer {
    #[default]
    Default,
    Enemy,
}

/// Create collision layers for enemies (collide with everything except other enemies)
pub fn enemy_collision_layers() -> CollisionLayers {
    let mut layers = CollisionLayers::new(GameLayer::Enemy, LayerMask::ALL);
    layers.filters.remove(GameLayer::Enemy);
    layers
}

#[derive(Component, Default)]
#[require(
    Name,
    Speed,
    Attack,
    Textured,
    Health,
    RigidBody::Dynamic,
    Hardness,
    LinearVelocity
)]
pub struct Enemy {}
