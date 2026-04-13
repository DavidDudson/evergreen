# Biome Layers, Decorations & Per-Biome Tilesets

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Restructure rendering layers to Tiles => Trees => Decorations, add 10-15 biome-specific decorations per area, and use distinct tilesets for each biome (city, greenwood, darkwood).

**Architecture:** The current 6-layer system (Tilemap, SceneryTree, Npc, Player, SceneryBush, SceneryFlower) collapses to 3 content layers (Tilemap, SceneryTree, Decoration) all below the player. Bushes and flowers become decorations. A new `Biome` enum drives tileset selection and decoration pool selection. Decorations are small ground-clutter sprites (10-15 per area) chosen from biome-specific asset pools.

**Tech Stack:** Rust, Bevy 0.18, bevy_ecs_tilemap, PixelLab (asset generation)

---

## Design: Decoration Catalog

All sizes are relative to the **player** (32x64 px visual, occupies 1x2 tiles / 16x32 game units).

### Size Classes

| Class | Pixel Size | Tile Footprint | Visual Scale | Examples |
|-------|-----------|----------------|--------------|---------|
| Small | 16x16 | 1x1 | Ankle-height | Flowers, mushrooms, twigs, sacks |
| Medium | 24x24 | 1.5x1.5 | Knee-height | Bushes, ferns, crates, barrels |
| Large | 32x16 | 2x1 | Wide/flat | Fallen logs, spider webs, carts |

### Darkwood Decorations (alignment 76-100)

Dark, dangerous, decayed. Ground is mossy and damp.

| # | Name | Size | Collider | Rustleable | Notes |
|---|------|------|----------|------------|-------|
| 1 | Poison mushroom cluster | 16x16 | No | No | Purple/green glowing caps |
| 2 | Thorn bush | 24x24 | No | Yes | Dark red-brown thorny mass |
| 3 | Spider web (ground) | 32x16 | No | No | Translucent white, stretched flat |
| 4 | Dead branch | 24x16 | No | No | Gray-brown broken wood |
| 5 | Glowing fungus | 16x16 | No | No | Bioluminescent blue-green on a stump |
| 6 | Skull and bones | 16x16 | No | No | Bleached white, half-buried |
| 7 | Dark wilted flower | 16x16 | No | Yes | Purple-black drooping petals |

Asset paths: `sprites/scenery/decorations/darkwood/`

### Greenwood Decorations (alignment 26-75)

Lush, vibrant, alive. Classic fairy-tale forest floor.

| # | Name | Size | Collider | Rustleable | Notes |
|---|------|------|----------|------------|-------|
| 1 | Wildflower patch | 16x16 | No | Yes | Mixed yellow/purple blooms |
| 2 | Herb cluster | 16x16 | No | Yes | Low green leafy plants |
| 3 | Twig pile | 16x16 | No | No | Small crossed sticks |
| 4 | Berry bush | 24x24 | No | Yes | Green with red berries |
| 5 | Fern | 24x24 | No | Yes | Bright green fronds |
| 6 | Mossy rock | 24x24 | No | No | Gray stone with green moss |
| 7 | Fallen log | 24x16 | No | No | Brown bark, moss patches |

Asset paths: `sprites/scenery/decorations/greenwood/`

### City Decorations (alignment 1-25)

Civilized clutter. Trade goods, infrastructure, domestic items.

| # | Name | Size | Collider | Rustleable | Notes |
|---|------|------|----------|------------|-------|
| 1 | Wooden crate | 24x24 | No | No | Brown planks, iron bands |
| 2 | Barrel | 24x24 | No | No | Round, dark wood, metal rings |
| 3 | Hay bale | 24x24 | No | No | Golden yellow, bound with rope |
| 4 | Sack of goods | 16x16 | No | No | Tan cloth, tied at top |
| 5 | Flower pot | 16x16 | No | No | Terracotta with blooms |
| 6 | Wooden sign | 16x16 | No | No | Small directional post |
| 7 | Cart | 32x16 | No | No | Wooden handcart with wheel |

Asset paths: `sprites/scenery/decorations/city/`

### PixelLab Generation Notes

All decorations use the style guide at `research/art/pixellab_style_guide.md`. Key params:
- View: `low top-down`
- Outline: `single color outline`
- Shading: `basic shading`
- Detail: `medium detail`
- Canvas: pad ~50% beyond sprite size (e.g. 16px sprite on 32px canvas)
- Style suffix: "Warm earthy palette with hue-shifted shadows toward cool purple and highlights toward warm gold. Moderate saturation, clean readable forms, storybook fantasy RPG style. 16-bit pixel art aesthetic."

---

## Design: Per-Biome Tilesets

Each biome needs its own 16-tile Wang tileset (same corner-based layout as current `terrain_wang.webp`).

| Biome | Grass Appearance | Path Appearance | Mood |
|-------|-----------------|-----------------|------|
| **City** (1-25) | Short trimmed lawn, yellow-green | Cobblestone / flagstone, warm gray-brown | Tidy, civilized |
| **Greenwood** (26-75) | Lush vibrant grass, rich green | Packed dirt with grass tufts, warm brown | Current tileset (keep as-is) |
| **Darkwood** (76-100) | Dark moss/dead grass, muted olive-gray | Dark damp mud, near-black brown | Oppressive, decayed |

Asset paths:
- `sprites/terrain/terrain_wang_city.webp`
- `sprites/terrain/terrain_wang.webp` (existing -- becomes greenwood)
- `sprites/terrain/terrain_wang_darkwood.webp`

The tileset is selected per-area in `spawn_area_tilemap` based on alignment thresholds. All three use the same WANG_TO_ATLAS index mapping.

---

## File Structure

| Action | File | Responsibility |
|--------|------|----------------|
| Modify | `models/src/layer.rs` | Remove `SceneryBush`, `SceneryFlower`; add `Decoration` at z=5 |
| Create | `models/src/decoration.rs` | `Decoration` marker component, `Biome` enum, `DecorationKind` with size/asset metadata |
| Modify | `models/src/lib.rs` | Add `pub mod decoration` |
| Modify | `models/src/scenery.rs` | Remove `Rustleable`/`Rustling` if moved, or keep shared (both trees and decorations use it) |
| Modify | `level/src/scenery.rs` | Remove bush/flower spawning; trees only. Remove bush/flower constants and assets. |
| Create | `level/src/decorations.rs` | Biome-specific decoration spawning (10-15 per area), asset pool selection, spawn logic |
| Modify | `level/src/spawning.rs` | Call `decorations::spawn_area_decorations` alongside scenery. Select tileset by biome. |
| Modify | `level/src/plugin.rs` | Register decoration despawn system, add `decorations` module |
| Modify | `level/src/lib.rs` | Add `pub mod decorations` |

---

## Task 1: Update Layer Enum

**Files:**
- Modify: `models/src/layer.rs`

- [ ] **Step 1: Remove SceneryBush and SceneryFlower, add Decoration**

The new layer order puts all world content below the player. Decorations sit between trees and NPCs.

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Layer {
    Tilemap,
    SceneryTree,
    Decoration,
    Npc,
    Player,
    NpcLabel,
}

impl Layer {
    pub const fn z(self) -> u16 {
        match self {
            Self::Tilemap => 0,
            Self::SceneryTree => 3,
            Self::Decoration => 5,
            Self::Npc => 9,
            Self::Player => 10,
            Self::NpcLabel => 20,
        }
    }

    #[allow(clippy::as_conversions)]
    pub const fn z_f32(self) -> f32 {
        self.z() as f32
    }
}
```

- [ ] **Step 2: Fix all references to removed variants**

Search for `Layer::SceneryBush` and `Layer::SceneryFlower` across the codebase. These only appear in `level/src/scenery.rs` in `spawn_bush` and `spawn_flower`. These functions will be removed in Task 3, but to keep the build green now, temporarily change them to `Layer::Decoration`.

Run: `cargo build`
Expected: compiles with no errors.

- [ ] **Step 3: Commit**

```bash
git add models/src/layer.rs level/src/scenery.rs
git commit -m "refactor: collapse SceneryBush/SceneryFlower into Decoration layer"
```

---

## Task 2: Add Decoration Model

**Files:**
- Create: `models/src/decoration.rs`
- Modify: `models/src/lib.rs`

- [ ] **Step 1: Create the decoration module**

```rust
// models/src/decoration.rs

use bevy::prelude::Component;

/// Marker for decoration entities (ground clutter).
#[derive(Component, Default)]
pub struct Decoration;

/// Biome classification derived from area alignment.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
```

- [ ] **Step 2: Register the module**

In `models/src/lib.rs`, add:

```rust
pub mod decoration;
```

Run: `cargo build`
Expected: compiles.

- [ ] **Step 3: Commit**

```bash
git add models/src/decoration.rs models/src/lib.rs
git commit -m "feat: add Decoration marker component and Biome enum"
```

---

## Task 3: Refactor Scenery to Trees Only

**Files:**
- Modify: `level/src/scenery.rs`

This removes bush and flower spawning from scenery. Trees remain the only scenery type. The bush/flower density logic is removed -- it will be replaced by the decoration system in Task 4.

- [ ] **Step 1: Remove bush/flower constants, assets, and spawn functions**

Remove these constants:
- `BUSH_SIZE_PX`
- `FLOWER_SIZE_PX`
- `BUSH_ASSETS`
- `FLOWER_ASSETS`

Remove these functions:
- `spawn_bush`
- `spawn_flower`
- `spawn_by_hash`

Remove the `zone_thresholds` and `lerp_thresholds` functions (they managed the 3-type density distribution).

Remove the zone distance constants:
- `CORNER_CD`
- `EDGE_ED`
- `MID_ED`

- [ ] **Step 2: Simplify `spawn_area_scenery` to only spawn trees**

The tree spawning logic stays but simplifies. Trees spawn on grass tiles based on alignment-driven density with a simple threshold -- no need for the zone system since decorations will handle the rest.

```rust
/// Tree spawn probability (0-100 hash threshold) by biome.
/// City: sparse trees at edges. Greenwood: moderate. Darkwood: dense.
fn tree_threshold(alignment: u8, ed: u32) -> usize {
    // Only spawn trees away from the center (ed <= 6)
    if ed > 6 {
        return 0;
    }

    let base = match Biome::from_alignment(alignment) {
        Biome::City => 8,
        Biome::Greenwood => 30,
        Biome::Darkwood => 65,
    };

    // Denser near edges, sparser toward center
    if ed <= 2 {
        base + 20
    } else if ed <= 4 {
        base + 10
    } else {
        base
    }
}
```

Update `spawn_area_scenery` to use this single threshold:

```rust
fn spawn_area_scenery(
    commands: &mut Commands,
    asset_server: &AssetServer,
    area: &Area,
    area_pos: IVec2,
) {
    let tile_px = f32::from(TILE_SIZE_PX);
    let base = area_world_offset(area_pos);
    let base_offset_x = base.x - MAP_W_PX / 2.0;
    let base_offset_y = base.y - MAP_H_PX / 2.0;

    let ax = u32::from_ne_bytes(area_pos.x.to_ne_bytes());
    let ay = u32::from_ne_bytes(area_pos.y.to_ne_bytes());
    let area_seed = ax.wrapping_mul(2_654_435_761).wrapping_add(ay.wrapping_mul(1_013_904_223));

    for x in 0..MAP_WIDTH {
        for y in 0..MAP_HEIGHT {
            let xu = u32::from(x);
            let yu = u32::from(y);

            if area.terrain_at(xu, yu) != Some(Terrain::Grass) {
                continue;
            }

            let hash = tile_hash(xu, yu, area_seed) % 100;
            let ed = edge_dist(x, y);
            let threshold = tree_threshold(area.alignment, ed);

            if hash < threshold && clear_for_tree(area, xu, yu) {
                let variant = tile_hash(xu, yu, area_seed.wrapping_add(10)) % TREE_ASSETS.len();
                spawn_tree(commands, asset_server, TREE_ASSETS[variant], world_x, world_y);
            }
        }
    }
}
```

(Compute `world_x`/`world_y` the same way as currently done in the loop body.)

- [ ] **Step 3: Add `use` for Biome**

```rust
use models::decoration::Biome;
```

- [ ] **Step 4: Verify build**

Run: `cargo build`
Expected: compiles. Bushes/flowers no longer spawn.

- [ ] **Step 5: Commit**

```bash
git add level/src/scenery.rs
git commit -m "refactor: strip bush/flower spawning from scenery (trees only)"
```

---

## Task 4: Create Decoration Spawning System

**Files:**
- Create: `level/src/decorations.rs`
- Modify: `level/src/lib.rs`
- Modify: `level/src/spawning.rs`
- Modify: `level/src/plugin.rs`

This is the core task. Each area gets 10-15 decorations chosen from a biome-specific pool.

- [ ] **Step 1: Create `level/src/decorations.rs`**

```rust
use bevy::math::IVec2;
use bevy::prelude::*;
use models::decoration::{Biome, Decoration};
use models::layer::Layer;
use models::scenery::Rustleable;

use crate::area::{Area, MAP_HEIGHT, MAP_WIDTH};
use crate::spawning::{TILE_SIZE_PX, area_world_offset};
use crate::terrain::{tile_hash, Terrain};

// Pixel dimensions of one map area.
#[allow(clippy::as_conversions)]
const MAP_W_PX: f32 = MAP_WIDTH as f32 * TILE_SIZE_PX as f32;
#[allow(clippy::as_conversions)]
const MAP_H_PX: f32 = MAP_HEIGHT as f32 * TILE_SIZE_PX as f32;

const Y_SORT_SCALE: f32 = 0.001;

/// Target decoration count per area.
const MIN_DECORATIONS: usize = 10;
const MAX_DECORATIONS: usize = 15;

/// A decoration entry: asset path, pixel size, and whether it rustles.
struct DecorationDef {
    path: &'static str,
    width_px: f32,
    height_px: f32,
    rustleable: bool,
}

// -----------------------------------------------------------------------
// Biome asset pools
// -----------------------------------------------------------------------

const DARKWOOD_DECORATIONS: &[DecorationDef] = &[
    DecorationDef { path: "sprites/scenery/decorations/darkwood/poison_mushroom.webp", width_px: 16.0, height_px: 16.0, rustleable: false },
    DecorationDef { path: "sprites/scenery/decorations/darkwood/thorn_bush.webp", width_px: 24.0, height_px: 24.0, rustleable: true },
    DecorationDef { path: "sprites/scenery/decorations/darkwood/spider_web.webp", width_px: 32.0, height_px: 16.0, rustleable: false },
    DecorationDef { path: "sprites/scenery/decorations/darkwood/dead_branch.webp", width_px: 24.0, height_px: 16.0, rustleable: false },
    DecorationDef { path: "sprites/scenery/decorations/darkwood/glowing_fungus.webp", width_px: 16.0, height_px: 16.0, rustleable: false },
    DecorationDef { path: "sprites/scenery/decorations/darkwood/skull_bones.webp", width_px: 16.0, height_px: 16.0, rustleable: false },
    DecorationDef { path: "sprites/scenery/decorations/darkwood/dark_flower.webp", width_px: 16.0, height_px: 16.0, rustleable: true },
];

const GREENWOOD_DECORATIONS: &[DecorationDef] = &[
    DecorationDef { path: "sprites/scenery/decorations/greenwood/wildflower.webp", width_px: 16.0, height_px: 16.0, rustleable: true },
    DecorationDef { path: "sprites/scenery/decorations/greenwood/herb_cluster.webp", width_px: 16.0, height_px: 16.0, rustleable: true },
    DecorationDef { path: "sprites/scenery/decorations/greenwood/twig_pile.webp", width_px: 16.0, height_px: 16.0, rustleable: false },
    DecorationDef { path: "sprites/scenery/decorations/greenwood/berry_bush.webp", width_px: 24.0, height_px: 24.0, rustleable: true },
    DecorationDef { path: "sprites/scenery/decorations/greenwood/fern.webp", width_px: 24.0, height_px: 24.0, rustleable: true },
    DecorationDef { path: "sprites/scenery/decorations/greenwood/mossy_rock.webp", width_px: 24.0, height_px: 24.0, rustleable: false },
    DecorationDef { path: "sprites/scenery/decorations/greenwood/fallen_log.webp", width_px: 24.0, height_px: 16.0, rustleable: false },
];

const CITY_DECORATIONS: &[DecorationDef] = &[
    DecorationDef { path: "sprites/scenery/decorations/city/wooden_crate.webp", width_px: 24.0, height_px: 24.0, rustleable: false },
    DecorationDef { path: "sprites/scenery/decorations/city/barrel.webp", width_px: 24.0, height_px: 24.0, rustleable: false },
    DecorationDef { path: "sprites/scenery/decorations/city/hay_bale.webp", width_px: 24.0, height_px: 24.0, rustleable: false },
    DecorationDef { path: "sprites/scenery/decorations/city/sack.webp", width_px: 16.0, height_px: 16.0, rustleable: false },
    DecorationDef { path: "sprites/scenery/decorations/city/flower_pot.webp", width_px: 16.0, height_px: 16.0, rustleable: false },
    DecorationDef { path: "sprites/scenery/decorations/city/wooden_sign.webp", width_px: 16.0, height_px: 16.0, rustleable: false },
    DecorationDef { path: "sprites/scenery/decorations/city/cart.webp", width_px: 32.0, height_px: 16.0, rustleable: false },
];

// -----------------------------------------------------------------------
// Public API
// -----------------------------------------------------------------------

/// Spawn 10-15 decorations for a single area.
pub fn spawn_area_decorations(
    commands: &mut Commands,
    asset_server: &AssetServer,
    area: &Area,
    area_pos: IVec2,
) {
    let biome = Biome::from_alignment(area.alignment);
    let pool = match biome {
        Biome::City => CITY_DECORATIONS,
        Biome::Greenwood => GREENWOOD_DECORATIONS,
        Biome::Darkwood => DARKWOOD_DECORATIONS,
    };

    let tile_px = f32::from(TILE_SIZE_PX);
    let base = area_world_offset(area_pos);
    let base_offset_x = base.x - MAP_W_PX / 2.0;
    let base_offset_y = base.y - MAP_H_PX / 2.0;

    let ax = u32::from_ne_bytes(area_pos.x.to_ne_bytes());
    let ay = u32::from_ne_bytes(area_pos.y.to_ne_bytes());
    let area_seed = ax
        .wrapping_mul(2_654_435_761)
        .wrapping_add(ay.wrapping_mul(1_013_904_223));

    // Decoration-specific salt to avoid overlapping tree positions.
    let deco_seed = area_seed.wrapping_add(999);

    // Collect candidate grass tiles (not on edges where trees dominate).
    let mut candidates: Vec<(u32, u32)> = Vec::new();
    for x in 2..(MAP_WIDTH - 2) {
        for y in 2..(MAP_HEIGHT - 2) {
            let xu = u32::from(x);
            let yu = u32::from(y);
            if area.terrain_at(xu, yu) == Some(Terrain::Grass) {
                candidates.push((xu, yu));
            }
        }
    }

    // Deterministically shuffle candidates using area seed.
    let len = candidates.len();
    if len == 0 {
        return;
    }
    let mut rng = u64::from(deco_seed);
    for i in (1..len).rev() {
        rng = lcg(rng);
        #[allow(clippy::as_conversions)]
        let j = (rng % u64::try_from(i + 1).expect("i+1 fits u64")) as usize;
        candidates.swap(i, j);
    }

    // Pick decoration count (10-15) deterministically.
    rng = lcg(rng);
    #[allow(clippy::as_conversions)]
    let count = MIN_DECORATIONS
        + (rng % u64::try_from(MAX_DECORATIONS - MIN_DECORATIONS + 1)
            .expect("range fits u64")) as usize;
    let count = count.min(candidates.len());

    for (i, &(xu, yu)) in candidates.iter().take(count).enumerate() {
        // Pick decoration type from pool.
        let variant = tile_hash(xu, yu, deco_seed.wrapping_add(u32::try_from(i).unwrap_or(0)))
            % pool.len();
        let def = &pool[variant];

        let world_x = base_offset_x + f32::from(u16::try_from(xu).expect("xu fits u16")) * tile_px
            + tile_px / 2.0;
        let world_y = base_offset_y + f32::from(u16::try_from(yu).expect("yu fits u16")) * tile_px
            + tile_px / 2.0;

        spawn_decoration(commands, asset_server, def, world_x, world_y);
    }
}

/// Despawn all decorations on game exit.
pub fn despawn_decorations(
    mut commands: Commands,
    query: Query<Entity, With<Decoration>>,
) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

// -----------------------------------------------------------------------
// Private helpers
// -----------------------------------------------------------------------

fn spawn_decoration(
    commands: &mut Commands,
    asset_server: &AssetServer,
    def: &DecorationDef,
    world_x: f32,
    world_y: f32,
) {
    let z = Layer::Decoration.z_f32() - world_y * Y_SORT_SCALE;
    let mut entity = commands.spawn((
        Decoration,
        Sprite {
            image: asset_server.load(def.path),
            custom_size: Some(Vec2::new(def.width_px, def.height_px)),
            ..default()
        },
        Transform::from_xyz(world_x, world_y, z),
    ));
    if def.rustleable {
        entity.insert(Rustleable);
    }
}

fn lcg(state: u64) -> u64 {
    state
        .wrapping_mul(6_364_136_223_846_793_005)
        .wrapping_add(1_442_695_040_888_963_407)
}
```

- [ ] **Step 2: Register module in `level/src/lib.rs`**

Add:
```rust
pub mod decorations;
```

- [ ] **Step 3: Wire decoration spawning into `level/src/spawning.rs`**

In the `ensure_area_spawned` function, add the decoration spawn call after scenery:

```rust
use crate::decorations;

// Inside ensure_area_spawned, after the scenery line:
scenery::spawn_area_scenery_at(commands, asset_server, area, area_pos);
decorations::spawn_area_decorations(commands, asset_server, area, area_pos);
npcs::spawn_npc_for_area(commands, asset_server, atlas_layouts, area, area_pos);
```

- [ ] **Step 4: Register despawn in `level/src/plugin.rs`**

Add `crate::decorations` import and add `decorations::despawn_decorations` to the `OnExit(GameState::Playing)` system set:

```rust
use crate::decorations;

// In the OnExit systems:
(
    spawning::despawn_all_areas,
    scenery::despawn_scenery,
    decorations::despawn_decorations,
    npcs::despawn_npcs,
    galen::despawn_galen,
    exit::despawn_exit,
)
    .run_if(should_despawn_world),
```

- [ ] **Step 5: Verify build**

Run: `cargo build`
Expected: compiles. Decorations will show as missing-texture placeholders until assets are generated.

- [ ] **Step 6: Commit**

```bash
git add level/src/decorations.rs level/src/lib.rs level/src/spawning.rs level/src/plugin.rs
git commit -m "feat: add biome-specific decoration spawning system (10-15 per area)"
```

---

## Task 5: Per-Biome Tileset Selection

**Files:**
- Modify: `level/src/spawning.rs`

Currently `spawn_area_tilemap` loads a single `terrain_wang.webp`. This task makes it select the tileset based on the area's biome alignment.

- [ ] **Step 1: Add tileset path helper**

Add to `spawning.rs`:

```rust
use models::decoration::Biome;

/// Returns the Wang tileset asset path for the given area alignment.
fn terrain_tileset_path(alignment: u8) -> &'static str {
    match Biome::from_alignment(alignment) {
        Biome::City => "sprites/terrain/terrain_wang_city.webp",
        Biome::Greenwood => "sprites/terrain/terrain_wang.webp",
        Biome::Darkwood => "sprites/terrain/terrain_wang_darkwood.webp",
    }
}
```

- [ ] **Step 2: Use it in `spawn_area_tilemap`**

Replace the hardcoded texture load:

```rust
// Before:
let texture: Handle<Image> = asset_server.load("sprites/terrain/terrain_wang.webp");

// After:
let texture: Handle<Image> = asset_server.load(terrain_tileset_path(area.alignment));
```

- [ ] **Step 3: Verify build**

Run: `cargo build`
Expected: compiles. City and darkwood tilesets will show as missing-texture until generated.

- [ ] **Step 4: Commit**

```bash
git add level/src/spawning.rs
git commit -m "feat: select terrain tileset per biome (city/greenwood/darkwood)"
```

---

## Task 6: Clean Up Scenery Module

**Files:**
- Modify: `level/src/scenery.rs`

After Tasks 3-4, the scenery module should be clean. This task removes any dead code and ensures the module is focused on trees only.

- [ ] **Step 1: Remove unused imports and constants**

Remove any lingering references to bushes, flowers, or the old zone system. The file should contain only:
- Tree constants (`TREE_WIDTH_PX`, `TREE_HEIGHT_PX`, `TREE_ASSETS`, `TREE_COLLIDER_HALF`, `TREE_COLLIDER_OFFSET`)
- `Y_SORT_SCALE`, `RUSTLE_MAX_ANGLE`
- `MAP_W_PX`, `MAP_H_PX`
- `spawn_area_scenery_at` / `spawn_area_scenery` (trees only)
- `spawn_tree`, `clear_for_tree`, `tree_threshold`
- `animate_rustle` (shared -- decorations also use `Rustleable`)
- `despawn_scenery`
- `edge_dist` (used by tree threshold)

Remove `corner_dist_min` if no longer used.

- [ ] **Step 2: Verify the file is under 150 lines**

The tree-only scenery module should be significantly smaller than the current 319 lines.

Run: `cargo build && cargo clippy`
Expected: clean build, no warnings.

- [ ] **Step 3: Commit**

```bash
git add level/src/scenery.rs
git commit -m "chore: clean up scenery module after decoration extraction"
```

---

## Task 7: Generate Decoration Assets with PixelLab

**Files:**
- Create: 21 decoration sprites in `assets/sprites/scenery/decorations/{city,greenwood,darkwood}/`

This task generates all decoration sprites using PixelLab's `create_map_object` tool. Use the style guide at `research/art/pixellab_style_guide.md`.

- [ ] **Step 1: Create asset directories**

```bash
mkdir -p assets/sprites/scenery/decorations/{city,greenwood,darkwood}
```

- [ ] **Step 2: Generate darkwood decorations (7 sprites)**

Use `mcp__pixellab__create_map_object` for each. All use:
- `view: "low top-down"`
- `outline: "single color outline"`
- `shading: "basic shading"`
- `detail: "medium detail"`

Generate each with the appropriate `width`/`height` (canvas = 2x sprite size for padding, but PixelLab minimum is 32):

| Sprite | Canvas | Description |
|--------|--------|-------------|
| poison_mushroom | 32x32 | "Small cluster of three poisonous mushrooms, purple caps with green spots, dark forest floor. Warm earthy palette..." |
| thorn_bush | 48x48 | "Thorny bush with sharp dark red-brown branches, small wilted leaves. Warm earthy palette..." |
| spider_web | 48x32 | "Spider web stretched flat on ground, translucent white silk threads in radial pattern, dark background. Warm earthy palette..." |
| dead_branch | 48x32 | "Broken dead tree branch lying on ground, gray-brown bark peeling. Warm earthy palette..." |
| glowing_fungus | 32x32 | "Bioluminescent fungus growing on small tree stump, blue-green glow, dark surroundings. Warm earthy palette..." |
| skull_bones | 32x32 | "Small animal skull with scattered bones, bleached white, half-buried in dark earth. Warm earthy palette..." |
| dark_flower | 32x32 | "Single wilted flower with drooping purple-black petals, thin dark stem. Warm earthy palette..." |

- [ ] **Step 3: Generate greenwood decorations (7 sprites)**

| Sprite | Canvas | Description |
|--------|--------|-------------|
| wildflower | 32x32 | "Small patch of mixed wildflowers, yellow and purple blooms, green stems on grass. Warm earthy palette..." |
| herb_cluster | 32x32 | "Low cluster of green leafy herbs, basil-like leaves, rich forest green. Warm earthy palette..." |
| twig_pile | 32x32 | "Small pile of crossed brown twigs and sticks on grass. Warm earthy palette..." |
| berry_bush | 48x48 | "Small round green bush with bright red berries, lush leaves. Warm earthy palette..." |
| fern | 48x48 | "Green fern with curling fronds, bright forest green, low ground cover. Warm earthy palette..." |
| mossy_rock | 48x48 | "Small gray stone covered in green moss patches, rounded shape. Warm earthy palette..." |
| fallen_log | 48x32 | "Short fallen log lying on ground, brown bark with moss patches, mushroom growing on end. Warm earthy palette..." |

- [ ] **Step 4: Generate city decorations (7 sprites)**

| Sprite | Canvas | Description |
|--------|--------|-------------|
| wooden_crate | 48x48 | "Wooden storage crate with iron corner bands, brown planks, closed lid. Warm earthy palette..." |
| barrel | 48x48 | "Round wooden barrel with dark metal rings, aged brown wood. Warm earthy palette..." |
| hay_bale | 48x48 | "Bound hay bale, golden yellow straw with brown rope ties. Warm earthy palette..." |
| sack | 32x32 | "Small burlap sack tied at top with rope, tan cloth, slightly lumpy. Warm earthy palette..." |
| flower_pot | 32x32 | "Small terracotta pot with colorful flowers blooming, on ground. Warm earthy palette..." |
| wooden_sign | 32x32 | "Small wooden directional signpost, rough-cut plank on short post. Warm earthy palette..." |
| cart | 48x32 | "Small wooden handcart with single wheel, open cargo bed, brown wood. Warm earthy palette..." |

- [ ] **Step 5: Convert all generated images to webp**

Each PixelLab result comes as PNG. Convert:

```bash
for f in assets/sprites/scenery/decorations/**/*.png; do
  convert "$f" -quality 90 "${f%.png}.webp"
  rm "$f"
done
```

- [ ] **Step 6: Commit**

```bash
git add assets/sprites/scenery/decorations/
git commit -m "feat: add 21 biome-specific decoration sprites"
```

---

## Task 8: Generate Biome Tilesets with PixelLab

**Files:**
- Create: `assets/sprites/terrain/terrain_wang_city.webp`
- Create: `assets/sprites/terrain/terrain_wang_darkwood.webp`

Use `mcp__pixellab__create_topdown_tileset` to generate Wang tilesets that match the existing 16-tile layout.

- [ ] **Step 1: Generate city tileset**

Use `create_topdown_tileset`:
- `tile_size: {"width": 16, "height": 16}`
- `view: "high top-down"`
- `outline: "selective outline"`
- `shading: "basic shading"`
- `detail: "medium detail"`
- `lower_description: "cobblestone path, warm gray-brown flagstones, tidy and worn"`
- `upper_description: "short trimmed grass lawn, yellow-green, neat and civilized"`
- `transition_description: "grass growing between cobblestones, stone edges"`
- `transition_size: 0.5`

- [ ] **Step 2: Generate darkwood tileset**

Use `create_topdown_tileset`:
- Same tile_size and view
- `lower_description: "dark damp muddy path, near-black brown, wet and sunken"`
- `upper_description: "dark dead grass with moss patches, muted olive-gray, decayed"`
- `transition_description: "mud seeping into dying grass, dark earth"`
- `transition_size: 0.5`

- [ ] **Step 3: Arrange tiles in Wang order**

The generated tilesets may not match the exact WANG_TO_ATLAS mapping. Check the output tile order against `terrain::WANG_TO_ATLAS` and rearrange the spritesheet if needed. The existing mapping is:

```
WANG_TO_ATLAS = [6, 7, 10, 9, 2, 11, 4, 15, 5, 14, 1, 8, 3, 0, 13, 12]
```

Both new tilesets must use this same index-to-atlas mapping.

- [ ] **Step 4: Convert to webp and place**

```bash
convert terrain_wang_city.png -quality 90 assets/sprites/terrain/terrain_wang_city.webp
convert terrain_wang_darkwood.png -quality 90 assets/sprites/terrain/terrain_wang_darkwood.webp
```

- [ ] **Step 5: Commit**

```bash
git add assets/sprites/terrain/terrain_wang_city.webp assets/sprites/terrain/terrain_wang_darkwood.webp
git commit -m "feat: add city and darkwood Wang tilesets"
```

---

## Task 9: Verify End-to-End

- [ ] **Step 1: Build and run**

```bash
cargo build && trunk serve
```

Walk through several areas in-game. Verify:
1. City areas (alignment 1-25) show cobblestone paths, trimmed grass, and crates/barrels/carts
2. Greenwood areas (26-75) show the original tileset with wildflowers/ferns/herbs
3. Darkwood areas (76-100) show dark mud paths, dead grass, and mushrooms/thorns/webs
4. Trees still render correctly on all biomes
5. Decorations appear at the correct z-layer (below player, above tiles)
6. Rustleable decorations animate when walked through
7. Each area has 10-15 decorations

- [ ] **Step 2: Run lints**

```bash
cargo clippy && cargo fmt -- --check
```

- [ ] **Step 3: Final commit if any fixes needed**

```bash
git add -u
git commit -m "fix: address lint issues from biome decoration system"
```
