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

    /// Convert to a 1-100 area alignment scale based on the dominant faction.
    /// 1 = city, 50 = greenwood, 100 = darkwood.
    pub fn dominant_area_alignment(&self) -> u8 {
        if self.cities >= self.greenwoods && self.cities >= self.darkwoods {
            // City dominant: scale 0-10 → 1-25
            let t = f32::from(self.cities) / f32::from(MAX_ALIGNMENT);
            #[allow(clippy::as_conversions)]
            return 1 + (t * 24.0) as u8;
        }
        if self.darkwoods >= self.greenwoods {
            // Darkwood dominant: scale 0-10 → 76-100
            let t = f32::from(self.darkwoods) / f32::from(MAX_ALIGNMENT);
            #[allow(clippy::as_conversions)]
            return 76 + (t * 24.0) as u8;
        }
        // Greenwood dominant: scale 0-10 → 26-75
        let t = f32::from(self.greenwoods) / f32::from(MAX_ALIGNMENT);
        #[allow(clippy::as_conversions)]
        let val = 26 + (t * 49.0) as u8;
        val
    }
}
