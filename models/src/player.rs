use bevy::prelude::*;
use crate::speed::Speed;

/// Marker component for the player entity.
#[derive(Component)]
#[require(Speed)]
pub struct Player;
