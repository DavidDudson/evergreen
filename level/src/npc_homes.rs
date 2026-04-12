//! Randomised NPC home-area assignments.
//!
//! Each run, NPCs are distributed across non-origin areas using the world
//! seed for deterministic placement. The origin (0,0) is reserved for Galen.

use std::collections::{HashMap, HashSet};

use bevy::math::IVec2;
use bevy::prelude::*;

/// Identifies an NPC for home-area assignment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NpcKind {
    Mordred,
    Drizella,
    Bigby,
    Gothel,
    Morgana,
    Cadwallader,
}

/// All NPC kinds in the order they should be assigned.
const ALL_NPCS: [NpcKind; 6] = [
    NpcKind::Mordred,
    NpcKind::Drizella,
    NpcKind::Bigby,
    NpcKind::Gothel,
    NpcKind::Morgana,
    NpcKind::Cadwallader,
];

/// Maps each NPC to their home area, assigned randomly at world creation.
#[derive(Resource)]
pub struct NpcHomes {
    homes: HashMap<NpcKind, IVec2>,
}

impl NpcHomes {
    /// Assign NPCs to random non-origin areas from the pool of generated areas.
    pub fn assign(available_areas: &[IVec2], seed: u64) -> Self {
        // Filter out origin (reserved for Galen).
        let mut candidates: Vec<IVec2> = available_areas
            .iter()
            .copied()
            .filter(|pos| *pos != IVec2::ZERO)
            .collect();

        // Deterministic shuffle using the world seed.
        let mut rng = seed.wrapping_mul(7_046_029_254_386_353_131);
        for i in (1..candidates.len()).rev() {
            rng = lcg(rng);
            let j = usize::try_from(rng % u64::try_from(i + 1).expect("i+1 fits u64"))
                .expect("mod result fits usize");
            candidates.swap(i, j);
        }

        let mut homes = HashMap::new();
        for (i, npc) in ALL_NPCS.iter().enumerate() {
            if i < candidates.len() {
                homes.insert(*npc, candidates[i]);
            }
        }

        Self { homes }
    }

    /// Get the home area for an NPC.
    pub fn home(&self, npc: NpcKind) -> Option<IVec2> {
        self.homes.get(&npc).copied()
    }

    /// Check whether a given area contains any NPC.
    pub fn has_npc(&self, area: IVec2) -> bool {
        self.homes.values().any(|pos| *pos == area)
    }

    /// Get all areas that contain NPCs.
    pub fn npc_areas(&self) -> HashSet<IVec2> {
        self.homes.values().copied().collect()
    }

    /// Check whether a specific NPC is at the given area.
    pub fn npc_at(&self, area: IVec2, npc: NpcKind) -> bool {
        self.homes.get(&npc).copied() == Some(area)
    }
}

fn lcg(state: u64) -> u64 {
    state
        .wrapping_mul(6_364_136_223_846_793_005)
        .wrapping_add(1_442_695_040_888_963_407)
}
