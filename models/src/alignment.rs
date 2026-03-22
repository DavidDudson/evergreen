use bevy::prelude::*;
use serde::Deserialize;

const MAX_ALIGNMENT: u8 = 10;
const GREENWOODS_START: u8 = 5;

/// Identifies one of the three faction alignments.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize)]
pub enum AlignmentFaction {
    Greenwoods,
    Darkwoods,
    Cities,
}

/// Tracks the player's standing with each faction (0–10).
///
/// Greenwoods starts at 5; Darkwoods and Cities start at 0.
#[derive(Resource, Debug)]
pub struct PlayerAlignment {
    pub greenwoods: u8,
    pub darkwoods: u8,
    pub cities: u8,
}

impl Default for PlayerAlignment {
    fn default() -> Self {
        Self {
            greenwoods: GREENWOODS_START,
            darkwoods: 0,
            cities: 0,
        }
    }
}

impl PlayerAlignment {
    /// Grant +1 alignment to `faction`, capped at 10.
    pub fn grant(&mut self, faction: AlignmentFaction) {
        let val = match faction {
            AlignmentFaction::Greenwoods => &mut self.greenwoods,
            AlignmentFaction::Darkwoods => &mut self.darkwoods,
            AlignmentFaction::Cities => &mut self.cities,
        };
        *val = val.saturating_add(1).min(MAX_ALIGNMENT);
    }

    /// Current alignment score for `faction`.
    pub fn get(&self, faction: AlignmentFaction) -> u8 {
        match faction {
            AlignmentFaction::Greenwoods => self.greenwoods,
            AlignmentFaction::Darkwoods => self.darkwoods,
            AlignmentFaction::Cities => self.cities,
        }
    }
}
