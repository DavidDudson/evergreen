//! Generic tag-based placement system.
//!
//! Each placeable (decoration, scenery, creature, building) declares a
//! [`PlacementRequirement`] describing what terrain it can sit on. Each
//! terrain (water tile, ground tile) declares a [`TerrainTags`] description
//! of what it provides.
//!
//! The matcher is intentionally simple:
//!   1. All `requires` tags must be present in `terrain.tags`.
//!   2. No `forbids` tag may be in `terrain.tags`.
//!   3. Every `taint` on the terrain must be in `tolerates`.
//!
//! This lets us express things like:
//!   - lily pad: `requires=[water, still], forbids=[salt, hot]`
//!     (works on ponds + lakes, excluded from ocean / hot spring).
//!   - seashell: `requires=[water], tolerates=[salt]`
//!     (works on ocean even though ocean has the `salt` taint).
//!   - cactus: `requires=[ground, dry], forbids=[fertile]`.

/// A semantic tag attached to a terrain or used in a placement rule.
///
/// Kept as a static string so terrain-tag tables can be `const`. Use
/// short, lowercase, kebab-style identifiers. Add new tags as needed; no
/// central enum to maintain.
pub type Tag = &'static str;

// ---------------------------------------------------------------------------
// Common tag vocabulary -- expand as needed.
// ---------------------------------------------------------------------------

pub mod tag {
    use super::Tag;

    // Terrain types
    pub const GROUND: Tag = "ground";
    pub const WATER: Tag = "water";
    pub const SAND: Tag = "sand";
    pub const ROCK: Tag = "rock";
    pub const PATH: Tag = "path";

    // Water properties
    pub const STILL: Tag = "still";
    pub const FLOWING: Tag = "flowing";
    pub const SALT: Tag = "salt";
    pub const HOT: Tag = "hot";

    // Ground properties
    pub const FERTILE: Tag = "fertile";
    pub const DRY: Tag = "dry";

    // Edge / boundary qualifiers
    pub const EDGE: Tag = "edge";
    pub const SHORE: Tag = "shore";

    // Biome flavor
    pub const URBAN: Tag = "urban";
    pub const WILD: Tag = "wild";
    pub const DARK: Tag = "dark";
}

/// Tags that describe a terrain tile.
///
/// `tags` are positive descriptors -- "I am water", "I am still".
/// `taints` are caveats a placeable must explicitly tolerate to sit here
/// (e.g. an ocean tile is `tags: [water, salt, still]` AND `taints: [salt]`,
/// so ordinary water plants are excluded but salt-tolerant ones can pass).
#[derive(Debug, Clone, Copy)]
pub struct TerrainTags {
    pub tags: &'static [Tag],
    pub taints: &'static [Tag],
}

impl TerrainTags {
    pub const EMPTY: Self = Self {
        tags: &[],
        taints: &[],
    };

    pub const fn new(tags: &'static [Tag], taints: &'static [Tag]) -> Self {
        Self { tags, taints }
    }

    pub fn has(&self, tag: Tag) -> bool {
        self.tags.contains(&tag)
    }
}

/// Placement rule for a decoration / creature / scenery item.
///
/// All three lists are evaluated left-to-right with no precedence between
/// them; a placeable must satisfy all three.
#[derive(Debug, Clone, Copy)]
pub struct PlacementRequirement {
    pub requires: &'static [Tag],
    pub forbids: &'static [Tag],
    pub tolerates: &'static [Tag],
}

impl PlacementRequirement {
    pub const ANY: Self = Self {
        requires: &[],
        forbids: &[],
        tolerates: &[],
    };

    pub const fn requires(tags: &'static [Tag]) -> Self {
        Self {
            requires: tags,
            forbids: &[],
            tolerates: &[],
        }
    }

    pub const fn with_forbids(mut self, forbids: &'static [Tag]) -> Self {
        self.forbids = forbids;
        self
    }

    pub const fn with_tolerates(mut self, tolerates: &'static [Tag]) -> Self {
        self.tolerates = tolerates;
        self
    }

    /// True if this placement requirement is satisfied by `terrain`.
    pub fn allows(&self, terrain: &TerrainTags) -> bool {
        if !self.requires.iter().all(|t| terrain.tags.contains(t)) {
            return false;
        }
        if self.forbids.iter().any(|t| terrain.tags.contains(t)) {
            return false;
        }
        if !terrain.taints.iter().all(|t| self.tolerates.contains(t)) {
            return false;
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const POND: TerrainTags = TerrainTags::new(&[tag::WATER, tag::STILL], &[]);
    const OCEAN: TerrainTags =
        TerrainTags::new(&[tag::WATER, tag::STILL, tag::SALT], &[tag::SALT]);
    const HOTSPRING: TerrainTags =
        TerrainTags::new(&[tag::WATER, tag::STILL, tag::HOT], &[tag::HOT]);
    const GRASS: TerrainTags = TerrainTags::new(&[tag::GROUND, tag::FERTILE], &[]);

    #[test]
    fn lily_pad_allowed_on_pond() {
        let lily = PlacementRequirement::requires(&[tag::WATER, tag::STILL])
            .with_forbids(&[tag::SALT, tag::HOT]);
        assert!(lily.allows(&POND));
    }

    #[test]
    fn lily_pad_excluded_from_ocean_via_forbid() {
        let lily = PlacementRequirement::requires(&[tag::WATER, tag::STILL])
            .with_forbids(&[tag::SALT, tag::HOT]);
        assert!(!lily.allows(&OCEAN));
    }

    #[test]
    fn lily_pad_excluded_from_hotspring_via_taint() {
        let lily = PlacementRequirement::requires(&[tag::WATER, tag::STILL]);
        // even without an explicit forbid, the HOT taint excludes it.
        assert!(!lily.allows(&HOTSPRING));
    }

    #[test]
    fn salt_tolerant_seaweed_allowed_on_ocean() {
        let seaweed = PlacementRequirement::requires(&[tag::WATER])
            .with_tolerates(&[tag::SALT]);
        assert!(seaweed.allows(&OCEAN));
    }

    #[test]
    fn requires_ground_excluded_from_water() {
        let cactus = PlacementRequirement::requires(&[tag::GROUND]);
        assert!(!cactus.allows(&POND));
        assert!(cactus.allows(&GRASS));
    }

    #[test]
    fn any_allows_anywhere_untainted() {
        assert!(PlacementRequirement::ANY.allows(&POND));
        assert!(PlacementRequirement::ANY.allows(&GRASS));
        // ANY does NOT tolerate taints by default.
        assert!(!PlacementRequirement::ANY.allows(&OCEAN));
    }
}
