use crate::speed::Speed;
use bevy::prelude::*;

/// Marker component for the player entity.
#[derive(Component)]
#[require(Speed)]
pub struct Player;
