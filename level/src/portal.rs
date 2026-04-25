//! Portal entities -- one per map. Crossing a portal generates a fresh map
//! whose alignment matches the portal kind, transitions the player into it,
//! and despawns the previous map's content.
//!
//! Portal kind eligibility: each kind targets a specific alignment band; a
//! portal of kind K may only spawn in a host map M when |K.target -
//! M.alignment| <= [`PORTAL_BRIDGE_RANGE`]. This guarantees a smooth
//! biome-to-biome traversal -- a greenwood map can lead to a city or
//! mid-greenwood map but not directly to a darkwood realm; a deep-greenwood
//! map can lead onward to darkwood via the mirror.

use bevy::math::IVec2;
use bevy::prelude::*;

use crate::area::{AreaAlignment, NpcKind};

/// Maximum alignment difference between a host map and a portal's target
/// alignment for the portal to be eligible to spawn there.
pub const PORTAL_BRIDGE_RANGE: u8 = 30;

/// One of the three available portal flavours. Each binds to a target
/// biome and a signature NPC who appears beside it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PortalKind {
    /// Demonic summoning circle. Leads to a city map. Cadwallader appears
    /// beside it.
    DemonicCircle,
    /// Ornate mirror. Leads to a darkwood map. Bloody Mary appears beside
    /// it.
    Mirror,
    /// Fairy mushroom ring. Leads to a greenwood map. Mother Gothel appears
    /// beside it.
    MushroomCircle,
}

impl PortalKind {
    /// Approximate target biome alignment when the player crosses through.
    pub fn target_alignment(self) -> AreaAlignment {
        match self {
            Self::DemonicCircle => 5,
            Self::MushroomCircle => 30,
            Self::Mirror => 85,
        }
    }

    /// NPC that appears in the same area as this portal kind.
    pub fn signature_npc(self) -> NpcKind {
        match self {
            Self::DemonicCircle => NpcKind::Cadwallader,
            Self::Mirror => NpcKind::BloodyMary,
            Self::MushroomCircle => NpcKind::Gothel,
        }
    }

    /// Sprite path under `assets/sprites/portals/`.
    pub fn sprite_path(self) -> &'static str {
        match self {
            Self::DemonicCircle => "sprites/portals/demonic.webp",
            Self::Mirror => "sprites/portals/mirror.webp",
            Self::MushroomCircle => "sprites/portals/mushroom.webp",
        }
    }

    /// True when this portal can spawn in a map of the given alignment.
    pub fn eligible_for(self, map_alignment: AreaAlignment) -> bool {
        self.target_alignment().abs_diff(map_alignment) <= PORTAL_BRIDGE_RANGE
    }
}

/// All portal kinds, in eligibility-pick order.
pub const ALL_PORTAL_KINDS: [PortalKind; 3] = [
    PortalKind::DemonicCircle,
    PortalKind::MushroomCircle,
    PortalKind::Mirror,
];

/// Pick the portal kind that should spawn in a map. From the eligible kinds
/// (those whose target alignment is within [`PORTAL_BRIDGE_RANGE`] of the
/// map's alignment), pick deterministically by `seed`. Returns `None` if no
/// portal kind matches -- in which case the map has no portal.
pub fn pick_portal_kind(map_alignment: AreaAlignment, seed: u64) -> Option<PortalKind> {
    let eligible: Vec<PortalKind> = ALL_PORTAL_KINDS
        .iter()
        .copied()
        .filter(|p| p.eligible_for(map_alignment))
        .collect();
    if eligible.is_empty() {
        return None;
    }
    let idx = usize::try_from(seed % u64::try_from(eligible.len()).unwrap_or(1)).unwrap_or(0);
    eligible.get(idx).copied()
}

/// Per-map portal placement: which kind, which area it sits in, and the
/// approximate tile within the area.
#[derive(Debug, Clone, Copy)]
pub struct PortalPlacement {
    pub kind: PortalKind,
    pub area_pos: IVec2,
    pub tile_x: u32,
    pub tile_y: u32,
}

/// Component on the spawned portal sprite. Player overlap triggers the
/// map-transition system.
#[derive(Component, Debug, Clone, Copy)]
pub struct PortalEntity {
    pub kind: PortalKind,
}

/// Marker component for the portal's signature NPC so the NPC can be
/// despawned together with its portal during a map transition.
#[derive(Component, Debug, Clone, Copy)]
pub struct PortalNpc;

/// Message fired when the player overlaps a portal. The transition system
/// regenerates the world map at the portal's target alignment.
#[derive(Message, Clone, Copy)]
pub struct PortalCrossed {
    pub kind: PortalKind,
}

/// Player-portal overlap radius (square). Roughly half the sprite size so
/// the player has to step *into* the rune circle, not just clip its edge.
pub const PORTAL_OVERLAP_RADIUS_PX: f32 = 10.0;

/// Despawn every portal sprite. Called on map teardown.
pub fn despawn_portals(
    mut commands: Commands,
    portals: Query<Entity, With<PortalEntity>>,
) {
    for entity in &portals {
        commands.entity(entity).despawn();
    }
}

/// Resource holding the alignment of the next map the player is heading to.
/// Set when [`PortalCrossed`] fires; consumed by the `MapTransition`-state
/// regen system so the new map is generated at the portal's target.
#[derive(Resource, Default, Debug, Clone, Copy)]
pub struct PendingPortal {
    pub alignment: Option<crate::area::AreaAlignment>,
    /// Set to true by [`apply_map_transition`] and consumed by
    /// `regenerate_world`. Prevents `OnEnter(Playing)` from blowing away the
    /// portal-generated map and replacing it with a fresh greenwood one.
    pub just_transitioned: bool,
}

/// Update-Playing system: when a `PortalCrossed` arrives, store the target
/// alignment in [`PendingPortal`] and switch to `MapTransition` state. The
/// state's `OnExit(Playing)` chain tears down the world; `OnEnter
/// (MapTransition)` regenerates it.
pub fn enter_map_transition(
    mut events: MessageReader<PortalCrossed>,
    mut pending: ResMut<PendingPortal>,
    mut next: ResMut<NextState<models::game_states::GameState>>,
) {
    let Some(event) = events.read().last().copied() else {
        return;
    };
    pending.alignment = Some(event.kind.target_alignment());
    next.set(models::game_states::GameState::MapTransition);
}

/// `OnEnter(MapTransition)` system: regenerate [`crate::world::WorldMap`] at
/// the pending alignment and immediately switch back to `Playing`, which
/// fires the respawn chain.
pub fn apply_map_transition(
    mut pending: ResMut<PendingPortal>,
    mut world: ResMut<crate::world::WorldMap>,
    mut spawned: ResMut<crate::spawning::SpawnedAreas>,
    mut next: ResMut<NextState<models::game_states::GameState>>,
) {
    let alignment = pending.alignment.take().unwrap_or(world.alignment);
    let new_seed: u64 = rand::random();
    let next_id = crate::world::MapId(world.id.0.wrapping_add(1));
    *world = crate::world::WorldMap::generate(next_id, new_seed, alignment);
    spawned.0.clear();
    pending.just_transitioned = true;
    next.set(models::game_states::GameState::Playing);
}
