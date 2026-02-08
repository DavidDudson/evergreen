use crate::hardness::Hardness;
use avian2d::prelude::{Collider, RigidBody};
use bevy::prelude::Component;

#[derive(Component, Default)]
#[require(RigidBody::Static, Collider, Hardness)]
pub struct Scenery;
