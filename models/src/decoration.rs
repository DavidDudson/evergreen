use bevy::prelude::Component;

/// Marker for decoration entities (ground clutter).
#[derive(Component, Default)]
pub struct Decoration;

/// Biome classification derived from area alignment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Biome {
    City,
    Greenwood,
    Darkwood,
}

impl Biome {
    /// Classify an alignment value (1-100) into a biome.
    pub fn from_alignment(alignment: u8) -> Self {
        match alignment {
            1..=25 => Self::City,
            26..=75 => Self::Greenwood,
            _ => Self::Darkwood,
        }
    }
}
