# Biome Blending, Dense Trees & Stump Reveal Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add biome border blending, regenerate trees at 48x64 with biome variants and trunk-only colliders, unify z-ordering via y-sort, and add a crossfade stump-reveal system for tall entities.

**Architecture:** A shared `blended_alignment` function computes per-tile effective alignment by lerping toward neighbor areas. A new `Revealable` component with child sprites handles the crossfade stump mechanic. All world entities (trees, decorations, NPCs, player) share a single `Layer::World` z-base with y-sort offsets, fixing the z-ordering bug.

**Tech Stack:** Rust, Bevy 0.18, bevy_ecs_tilemap, PixelLab (asset generation)

---

## File Structure

| Action | File | Responsibility |
|--------|------|----------------|
| Modify | `models/src/layer.rs` | Replace SceneryTree/Decoration/Npc/Player with single `World` layer |
| Create | `models/src/reveal.rs` | `Revealable`, `RevealState`, `FullSprite`, `StumpSprite` components |
| Modify | `models/src/lib.rs` | Add `pub mod reveal` |
| Create | `level/src/blending.rs` | `blended_alignment()` function, blend zone constants |
| Modify | `level/src/lib.rs` | Add `pub mod blending` |
| Modify | `level/src/scenery.rs` | New tree sizes/assets/colliders, biome variants, use blended alignment |
| Modify | `level/src/decorations.rs` | Use blended alignment for pool selection |
| Modify | `level/src/spawning.rs` | Pass `WorldMap` to scenery/decorations, use blended alignment for tileset |
| Create | `level/src/reveal.rs` | Stump reveal detection and crossfade system |
| Modify | `level/src/plugin.rs` | Register reveal systems |
| Modify | `level/src/npcs.rs` | Use `Layer::World` with y-sort |
| Modify | `player/src/spawning.rs` | Use `Layer::World` with y-sort |
| Create | `player/src/y_sort.rs` | Per-frame player z update from y-position |
| Modify | `player/src/plugin.rs` | Register y-sort system |
| Modify | `player/src/lib.rs` | Add `pub mod y_sort` |

---

### Task 1: Simplify Layer Enum to World Layer

**Files:**
- Modify: `models/src/layer.rs`

- [ ] **Step 1: Replace entity-specific layers with a single `World` layer**

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Layer {
    Tilemap,
    World,
    NpcLabel,
}

impl Layer {
    pub const fn z(self) -> u16 {
        match self {
            Self::Tilemap => 0,
            Self::World => 5,
            Self::NpcLabel => 20,
        }
    }

    #[allow(clippy::as_conversions)]
    pub const fn z_f32(self) -> f32 {
        self.z() as f32
    }
}
```

- [ ] **Step 2: Fix all compilation errors from removed variants**

Search for `Layer::SceneryTree`, `Layer::Decoration`, `Layer::Npc`, `Layer::Player` across the workspace. Replace all with `Layer::World`. Files affected:
- `level/src/scenery.rs` -- `Layer::SceneryTree` -> `Layer::World`
- `level/src/decorations.rs` -- `Layer::Decoration` -> `Layer::World`
- `level/src/npcs.rs` -- `Layer::Npc` references -> `Layer::World`
- `player/src/spawning.rs` -- `Layer::Player` -> `Layer::World`

In `level/src/npcs.rs`, change the constant:
```rust
// Before:
const NPC_Z: f32 = Layer::Npc.z_f32();
// After -- remove the constant entirely, NPC z will be computed with y-sort in a later step
```
For now, temporarily use `Layer::World.z_f32()` where `NPC_Z` was used.

- [ ] **Step 3: Verify build**

Run: `cargo build`

- [ ] **Step 4: Commit**

```bash
git add models/src/layer.rs level/src/scenery.rs level/src/decorations.rs level/src/npcs.rs player/src/spawning.rs
git commit -m "refactor: unify entity layers into single World layer for y-sort"
```

---

### Task 2: Add Y-Sort to Player and NPCs

**Files:**
- Create: `player/src/y_sort.rs`
- Modify: `player/src/lib.rs`
- Modify: `player/src/plugin.rs`
- Modify: `player/src/spawning.rs`
- Modify: `level/src/npcs.rs`

The y-sort scale constant `0.001` is already used in scenery/decorations. Player and NPCs need the same treatment -- their z must update every frame based on y-position.

- [ ] **Step 1: Create `player/src/y_sort.rs`**

```rust
use bevy::prelude::*;
use models::layer::Layer;

use crate::spawning::Player;

/// Y-sort scale -- must match the value used in level scenery/decorations.
const Y_SORT_SCALE: f32 = 0.001;

/// Update the player's z-position each frame for correct y-sort ordering.
pub fn update_player_z(mut query: Query<&mut Transform, With<Player>>) {
    let Ok(mut tf) = query.single_mut() else {
        return;
    };
    tf.translation.z = Layer::World.z_f32() - tf.translation.y * Y_SORT_SCALE;
}
```

- [ ] **Step 2: Register in `player/src/lib.rs`**

Add: `pub mod y_sort;`

- [ ] **Step 3: Register the system in `player/src/plugin.rs`**

Add `y_sort::update_player_z` to the `Update` systems that run in `GameState::Playing`, after `move_player`:

```rust
use crate::y_sort;
// In the Update system set:
y_sort::update_player_z,
```

- [ ] **Step 4: Fix player spawn z**

In `player/src/spawning.rs`, change the spawn transform to use y-sorted z:

```rust
// Before:
Transform::from_xyz(
    area_world_offset(world.current).x,
    area_world_offset(world.current).y,
    Layer::Player.z_f32(),
),
// After:
Transform::from_xyz(
    area_world_offset(world.current).x,
    area_world_offset(world.current).y,
    Layer::World.z_f32(),
),
```

(The y_sort system will correct it on the first frame.)

- [ ] **Step 5: Fix NPC spawn z with y-sort**

In `level/src/npcs.rs`, update `tile_world_pos` to use y-sorted z:

```rust
const Y_SORT_SCALE: f32 = 0.001;

fn tile_world_pos(tx: u16, ty: u16, base: Vec2) -> Vec3 {
    let tile_px = f32::from(TILE_SIZE_PX);
    let offset_x = base.x - (f32::from(MAP_WIDTH) * tile_px) / 2.0;
    let offset_y = base.y - (f32::from(MAP_HEIGHT) * tile_px) / 2.0;
    let world_y = offset_y + f32::from(ty) * tile_px + tile_px / 2.0;
    Vec3::new(
        offset_x + f32::from(tx) * tile_px + tile_px / 2.0,
        world_y,
        Layer::World.z_f32() - world_y * Y_SORT_SCALE,
    )
}
```

Remove the old `NPC_Z` constant if it still exists.

- [ ] **Step 6: Add NPC y-sort update system**

NPCs wander, so they need per-frame z updates too. In `level/src/npc_wander.rs` or `level/src/npcs.rs` (whichever has the wander system), add z-update after position changes. Alternatively, add a dedicated system in `level/src/npcs.rs`:

```rust
const Y_SORT_SCALE: f32 = 0.001;

pub fn update_npc_z(mut query: Query<&mut Transform, With<EventNpc>>) {
    for mut tf in &mut query {
        tf.translation.z = Layer::World.z_f32() - tf.translation.y * Y_SORT_SCALE;
    }
}
```

Register this in `level/src/plugin.rs` in the `Update` systems.

- [ ] **Step 7: Verify build**

Run: `cargo build`

- [ ] **Step 8: Commit**

```bash
git add player/src/y_sort.rs player/src/lib.rs player/src/plugin.rs player/src/spawning.rs level/src/npcs.rs level/src/plugin.rs
git commit -m "feat: add per-frame y-sort z-ordering for player and NPCs"
```

---

### Task 3: Create Biome Blending Module

**Files:**
- Create: `level/src/blending.rs`
- Modify: `level/src/lib.rs`

- [ ] **Step 1: Create `level/src/blending.rs`**

```rust
use bevy::math::IVec2;

use crate::area::{Direction, MAP_HEIGHT, MAP_WIDTH};
use crate::world::WorldMap;

/// Horizontal blend width in tiles (20% of 32).
const BLEND_W: u32 = 6;
/// Vertical blend width in tiles (20% of 18).
const BLEND_H: u32 = 4;

/// Compute the effective biome alignment for a tile at `(x, y)` within an area,
/// blending toward neighbor areas near the borders.
///
/// Returns the area's own alignment if the tile is outside all blend zones
/// or if no neighbor exists in the relevant direction.
pub fn blended_alignment(
    area_alignment: u8,
    x: u32,
    y: u32,
    area_pos: IVec2,
    world: &WorldMap,
) -> u8 {
    let w = u32::from(MAP_WIDTH);
    let h = u32::from(MAP_HEIGHT);

    // Find the strongest neighbor influence.
    let mut best_t: f32 = 0.0;
    let mut best_neighbor_align: Option<u8> = None;

    // West edge
    if x < BLEND_W {
        let t = 1.0 - (x as f32 / BLEND_W as f32);
        if let Some(align) = neighbor_alignment(area_pos, Direction::West, world) {
            if t > best_t {
                best_t = t;
                best_neighbor_align = Some(align);
            }
        }
    }

    // East edge
    let dist_right = w.saturating_sub(1).saturating_sub(x);
    if dist_right < BLEND_W {
        let t = 1.0 - (dist_right as f32 / BLEND_W as f32);
        if let Some(align) = neighbor_alignment(area_pos, Direction::East, world) {
            if t > best_t {
                best_t = t;
                best_neighbor_align = Some(align);
            }
        }
    }

    // South edge
    if y < BLEND_H {
        let t = 1.0 - (y as f32 / BLEND_H as f32);
        if let Some(align) = neighbor_alignment(area_pos, Direction::South, world) {
            if t > best_t {
                best_t = t;
                best_neighbor_align = Some(align);
            }
        }
    }

    // North edge
    let dist_top = h.saturating_sub(1).saturating_sub(y);
    if dist_top < BLEND_H {
        let t = 1.0 - (dist_top as f32 / BLEND_H as f32);
        if let Some(align) = neighbor_alignment(area_pos, Direction::North, world) {
            if t > best_t {
                best_t = t;
                best_neighbor_align = Some(align);
            }
        }
    }

    match best_neighbor_align {
        Some(neighbor) => lerp_alignment(area_alignment, neighbor, best_t * 0.5),
        None => area_alignment,
    }
}

fn neighbor_alignment(area_pos: IVec2, dir: Direction, world: &WorldMap) -> Option<u8> {
    let neighbor_pos = area_pos + dir.grid_offset();
    world.get_area(neighbor_pos).map(|a| a.alignment)
}

#[allow(clippy::as_conversions)]
fn lerp_alignment(a: u8, b: u8, t: f32) -> u8 {
    let result = f32::from(a) + (f32::from(b) - f32::from(a)) * t;
    result.round().clamp(1.0, 100.0) as u8
}
```

- [ ] **Step 2: Register in `level/src/lib.rs`**

Add: `pub mod blending;`

- [ ] **Step 3: Verify build**

Run: `cargo build`

- [ ] **Step 4: Commit**

```bash
git add level/src/blending.rs level/src/lib.rs
git commit -m "feat: add biome blending module for border alignment interpolation"
```

---

### Task 4: Apply Blending to Tileset Selection

**Files:**
- Modify: `level/src/spawning.rs`

Currently `spawn_area_tilemap` uses one tileset for the entire area. With blending, tiles in the blend zone may use a different biome's tileset. Since `bevy_ecs_tilemap` uses a single texture per tilemap, we can't mix tilesets within one tilemap. Instead, we use the blended alignment at the **area center** to pick the tileset -- this means an area that is mostly influenced by a neighbor will shift its entire tileset. Per-tile tileset mixing would require multiple overlapping tilemaps which is too complex for now.

- [ ] **Step 1: Update tileset selection to use center-blended alignment**

```rust
use crate::blending;

// In spawn_area_tilemap, replace:
let texture: Handle<Image> = asset_server.load(terrain_tileset_path(area.alignment));
// With:
let center_x = u32::from(MAP_WIDTH) / 2;
let center_y = u32::from(MAP_HEIGHT) / 2;
let effective_alignment = blending::blended_alignment(
    area.alignment,
    center_x,
    center_y,
    area_pos,
    world,
);
let texture: Handle<Image> = asset_server.load(terrain_tileset_path(effective_alignment));
```

- [ ] **Step 2: Verify build**

Run: `cargo build`

- [ ] **Step 3: Commit**

```bash
git add level/src/spawning.rs
git commit -m "feat: apply biome blending to tileset selection"
```

---

### Task 5: Apply Blending to Scenery and Decorations

**Files:**
- Modify: `level/src/scenery.rs`
- Modify: `level/src/decorations.rs`
- Modify: `level/src/spawning.rs`

Both scenery and decorations need the `WorldMap` reference and area position to compute per-tile blended alignment.

- [ ] **Step 1: Update scenery function signatures to accept `WorldMap` and `area_pos`**

In `level/src/scenery.rs`, update `spawn_area_scenery_at` and `spawn_area_scenery`:

```rust
use crate::blending;
use crate::world::WorldMap;

pub fn spawn_area_scenery_at(
    commands: &mut Commands,
    asset_server: &AssetServer,
    area: &Area,
    area_pos: IVec2,
    world: &WorldMap,
) {
    spawn_area_scenery(commands, asset_server, area, area_pos, world);
}
```

In the `spawn_area_scenery` loop body, compute blended alignment per tile:

```rust
let effective_alignment = blending::blended_alignment(
    area.alignment,
    xu,
    yu,
    area_pos,
    world,
);
let threshold = tree_threshold(effective_alignment, ed);
```

Also use blended alignment to select tree variant pool (biome-specific variants -- will be added in Task 7).

- [ ] **Step 2: Update decorations to use blended alignment**

In `level/src/decorations.rs`, add parameters and use blended alignment:

```rust
use crate::blending;
use crate::world::WorldMap;

pub fn spawn_area_decorations(
    commands: &mut Commands,
    asset_server: &AssetServer,
    area: &Area,
    area_pos: IVec2,
    world: &WorldMap,
) {
    // ... existing setup ...

    for (i, &(xu, yu)) in candidates.iter().take(count).enumerate() {
        let effective_alignment = blending::blended_alignment(
            area.alignment,
            xu,
            yu,
            area_pos,
            world,
        );
        let biome = Biome::from_alignment(effective_alignment);
        let pool = match biome {
            Biome::City => CITY_DECORATIONS,
            Biome::Greenwood => GREENWOOD_DECORATIONS,
            Biome::Darkwood => DARKWOOD_DECORATIONS,
        };
        // ... rest of spawn logic using this pool ...
    }
}
```

Move the pool selection inside the per-decoration loop (it was previously computed once for the whole area).

- [ ] **Step 3: Update spawning.rs call sites**

In `level/src/spawning.rs`, pass `world` to the updated functions:

```rust
scenery::spawn_area_scenery_at(commands, asset_server, area, area_pos, world);
decorations::spawn_area_decorations(commands, asset_server, area, area_pos, world);
```

- [ ] **Step 4: Verify build**

Run: `cargo build`

- [ ] **Step 5: Commit**

```bash
git add level/src/scenery.rs level/src/decorations.rs level/src/spawning.rs
git commit -m "feat: apply biome blending to scenery and decoration spawning"
```

---

### Task 6: Add Revealable Components

**Files:**
- Create: `models/src/reveal.rs`
- Modify: `models/src/lib.rs`

- [ ] **Step 1: Create `models/src/reveal.rs`**

```rust
use bevy::prelude::Component;

/// Marks an entity that can reveal its stump when the player walks behind it.
#[derive(Component)]
pub struct Revealable {
    /// How far above the entity base the canopy/top extends (pixels).
    pub canopy_height_px: f32,
    /// Half-width of the reveal trigger zone (pixels).
    pub half_width_px: f32,
}

/// Current state of the reveal transition.
#[derive(Component, Default)]
pub enum RevealState {
    #[default]
    Full,
    /// Transitioning to stump. Progress 0.0 (full) -> 1.0 (stump).
    Revealing(f32),
    Revealed,
    /// Transitioning back to full. Progress 0.0 (stump) -> 1.0 (full).
    Hiding(f32),
}

/// Marker for the full (canopy) sprite child.
#[derive(Component)]
pub struct FullSprite;

/// Marker for the stump sprite child.
#[derive(Component)]
pub struct StumpSprite;
```

- [ ] **Step 2: Register in `models/src/lib.rs`**

Add: `pub mod reveal;`

- [ ] **Step 3: Verify build**

Run: `cargo build`

- [ ] **Step 4: Commit**

```bash
git add models/src/reveal.rs models/src/lib.rs
git commit -m "feat: add Revealable, RevealState, FullSprite, StumpSprite components"
```

---

### Task 7: Regenerate Tree Assets and Update Scenery

**Files:**
- Modify: `level/src/scenery.rs`
- Delete: `assets/sprites/scenery/trees/tree_oak.webp`, `tree_pine.webp`
- Create: 16 new tree assets in `assets/sprites/scenery/trees/{city,greenwood,darkwood}/`

- [ ] **Step 1: Generate tree assets via PixelLab**

Use `mcp__pixellab__create_map_object` in batches of 4 (respecting rate limits, ~60s between batches).

**Full trees** (48x64, `view: "low top-down"`, `outline: "single color outline"`, `shading: "basic shading"`, `detail: "medium detail"`):

| Biome | Name | Description |
|-------|------|-------------|
| City | tree_city_ornamental | "Ornamental tree with neatly trimmed round canopy, short trunk, civilized garden tree. Warm earthy palette..." |
| City | tree_city_fruit | "Small fruit tree with round canopy and visible red fruits, neat garden tree. Warm earthy palette..." |
| Greenwood | tree_green_oak | "Broad leafy oak tree with thick trunk and wide lush green canopy, forest tree. Warm earthy palette..." |
| Greenwood | tree_green_birch | "Slender birch tree with white bark and delicate green leaf canopy. Warm earthy palette..." |
| Greenwood | tree_green_maple | "Full maple tree with rich green rounded canopy and brown trunk. Warm earthy palette..." |
| Darkwood | tree_dark_gnarled | "Twisted gnarled tree with dark bark, sparse withered leaves, creepy dark forest. Warm earthy palette..." |
| Darkwood | tree_dark_dead | "Dead tree with bare branches, gray cracked bark, no leaves, haunted forest. Warm earthy palette..." |
| Darkwood | tree_dark_willow | "Dark weeping willow with drooping black-green branches, eerie and sad. Warm earthy palette..." |

**Stumps** (48x24, same style params):

For each tree above, generate a matching stump: "Trunk and roots only of [same description], no canopy, cut off at waist height, just the base portion."

All append style suffix: "Warm earthy palette with hue-shifted shadows toward cool purple and highlights toward warm gold. Moderate saturation, clean readable forms, storybook fantasy RPG style. 16-bit pixel art."

Convert all to webp. Place in:
```
assets/sprites/scenery/trees/city/
assets/sprites/scenery/trees/greenwood/
assets/sprites/scenery/trees/darkwood/
```

Delete old `tree_oak.webp` and `tree_pine.webp`.

- [ ] **Step 2: Update scenery constants and tree spawning**

Replace the tree constants and assets in `level/src/scenery.rs`:

```rust
const TREE_WIDTH_PX: f32 = 48.0;
const TREE_HEIGHT_PX: f32 = 64.0;
const STUMP_HEIGHT_PX: f32 = 24.0;

// Trunk-only collider: 1x1 tile at the base.
const TREE_COLLIDER_HALF: Vec2 = Vec2::new(8.0, 8.0);
const TREE_COLLIDER_OFFSET: Vec2 = Vec2::new(0.0, 4.0);

struct TreeDef {
    full_path: &'static str,
    stump_path: &'static str,
}

const CITY_TREES: &[TreeDef] = &[
    TreeDef { full_path: "sprites/scenery/trees/city/tree_city_ornamental.webp", stump_path: "sprites/scenery/trees/city/tree_city_ornamental_stump.webp" },
    TreeDef { full_path: "sprites/scenery/trees/city/tree_city_fruit.webp", stump_path: "sprites/scenery/trees/city/tree_city_fruit_stump.webp" },
];

const GREENWOOD_TREES: &[TreeDef] = &[
    TreeDef { full_path: "sprites/scenery/trees/greenwood/tree_green_oak.webp", stump_path: "sprites/scenery/trees/greenwood/tree_green_oak_stump.webp" },
    TreeDef { full_path: "sprites/scenery/trees/greenwood/tree_green_birch.webp", stump_path: "sprites/scenery/trees/greenwood/tree_green_birch_stump.webp" },
    TreeDef { full_path: "sprites/scenery/trees/greenwood/tree_green_maple.webp", stump_path: "sprites/scenery/trees/greenwood/tree_green_maple_stump.webp" },
];

const DARKWOOD_TREES: &[TreeDef] = &[
    TreeDef { full_path: "sprites/scenery/trees/darkwood/tree_dark_gnarled.webp", stump_path: "sprites/scenery/trees/darkwood/tree_dark_gnarled_stump.webp" },
    TreeDef { full_path: "sprites/scenery/trees/darkwood/tree_dark_dead.webp", stump_path: "sprites/scenery/trees/darkwood/tree_dark_dead_stump.webp" },
    TreeDef { full_path: "sprites/scenery/trees/darkwood/tree_dark_willow.webp", stump_path: "sprites/scenery/trees/darkwood/tree_dark_willow_stump.webp" },
];
```

- [ ] **Step 3: Update tree_threshold for denser forests**

```rust
fn tree_threshold(alignment: u8, ed: u32) -> usize {
    if ed > 6 {
        return 0;
    }
    let base: usize = match Biome::from_alignment(alignment) {
        Biome::City => 5,
        Biome::Greenwood => 45,
        Biome::Darkwood => 80,
    };
    if ed <= 2 {
        base + 15
    } else if ed <= 4 {
        base + 10
    } else {
        base
    }
}
```

- [ ] **Step 4: Update `clear_for_tree` for 1x1 trunk footprint**

```rust
/// Returns `true` when the trunk tile at `(xu, yu)` is on grass.
fn clear_for_tree(area: &Area, xu: u32, yu: u32) -> bool {
    area.terrain_at(xu, yu) == Some(Terrain::Grass)
}
```

- [ ] **Step 5: Update `spawn_tree` to use biome variant pool and child sprites**

Select tree pool based on blended alignment. Spawn tree with `Revealable` component and two child sprites (full + stump):

```rust
use bevy::sprite::Anchor;
use models::reveal::{FullSprite, Revealable, RevealState, StumpSprite};

fn tree_pool(alignment: u8) -> &'static [TreeDef] {
    match Biome::from_alignment(alignment) {
        Biome::City => CITY_TREES,
        Biome::Greenwood => GREENWOOD_TREES,
        Biome::Darkwood => DARKWOOD_TREES,
    }
}

fn spawn_tree(
    commands: &mut Commands,
    asset_server: &AssetServer,
    def: &TreeDef,
    world_x: f32,
    world_y: f32,
) {
    let z = Layer::World.z_f32() - world_y * Y_SORT_SCALE;
    let tree_entity = commands
        .spawn((
            Scenery,
            SceneryCollider {
                half_extents: TREE_COLLIDER_HALF,
                center_offset: TREE_COLLIDER_OFFSET,
            },
            Revealable {
                canopy_height_px: TREE_HEIGHT_PX - STUMP_HEIGHT_PX,
                half_width_px: TREE_WIDTH_PX / 2.0,
            },
            RevealState::default(),
            Transform::from_xyz(world_x, world_y, z),
            Visibility::default(),
        ))
        .id();

    // Full tree sprite (child)
    let full_child = commands
        .spawn((
            FullSprite,
            Sprite {
                image: asset_server.load(def.full_path),
                custom_size: Some(Vec2::new(TREE_WIDTH_PX, TREE_HEIGHT_PX)),
                anchor: Anchor::BottomCenter,
                ..default()
            },
            Transform::IDENTITY,
        ))
        .id();

    // Stump sprite (child, starts invisible)
    let stump_child = commands
        .spawn((
            StumpSprite,
            Sprite {
                image: asset_server.load(def.stump_path),
                custom_size: Some(Vec2::new(TREE_WIDTH_PX, STUMP_HEIGHT_PX)),
                anchor: Anchor::BottomCenter,
                color: Color::srgba(1.0, 1.0, 1.0, 0.0),
                ..default()
            },
            Transform::IDENTITY,
        ))
        .id();

    commands.entity(tree_entity).add_children(&[full_child, stump_child]);
}
```

Note: The `color` field on stump uses `Color::srgba` -- since `clippy::disallowed_methods` bans inline color constructors, add this color as a constant in `models/src/palette.rs`:

```rust
pub const TRANSPARENT: Color = Color::srgba(1.0, 1.0, 1.0, 0.0);
pub const OPAQUE_WHITE: Color = Color::srgba(1.0, 1.0, 1.0, 1.0);
```

- [ ] **Step 6: Update spawn loop to use blended alignment for tree pool**

In `spawn_area_scenery`, after computing `effective_alignment`:

```rust
let pool = tree_pool(effective_alignment);
let variant = tile_hash(xu, yu, area_seed.wrapping_add(10)) % pool.len();
let def = &pool[variant];
spawn_tree(commands, asset_server, def, world_x, world_y);
```

- [ ] **Step 7: Verify build**

Run: `cargo build`

- [ ] **Step 8: Commit**

```bash
git add assets/sprites/scenery/trees/ level/src/scenery.rs models/src/palette.rs
git rm assets/sprites/scenery/trees/tree_oak.webp assets/sprites/scenery/trees/tree_pine.webp
git commit -m "feat: regenerate trees at 48x64 with biome variants and stump sprites"
```

---

### Task 8: Implement Stump Reveal System

**Files:**
- Create: `level/src/reveal.rs`
- Modify: `level/src/lib.rs`
- Modify: `level/src/plugin.rs`

- [ ] **Step 1: Create `level/src/reveal.rs`**

```rust
use bevy::prelude::*;
use models::reveal::{FullSprite, Revealable, RevealState, StumpSprite};
use player::spawning::Player;

/// Distance north of entity base that triggers reveal (pixels).
const REVEAL_TRIGGER_PX: f32 = 16.0;

/// Duration of the crossfade transition (seconds).
const REVEAL_DURATION_SECS: f32 = 0.3;

/// Detect when the player is behind revealable entities and trigger transitions.
pub fn detect_reveals(
    player_q: Query<&Transform, With<Player>>,
    mut revealables: Query<(&Transform, &Revealable, &mut RevealState), Without<Player>>,
) {
    let Ok(player_tf) = player_q.single() else {
        return;
    };
    let pp = player_tf.translation.truncate();

    for (tf, revealable, mut state) in &mut revealables {
        let base = tf.translation.truncate();
        let behind = pp.y > base.y
            && pp.y < base.y + REVEAL_TRIGGER_PX
            && (pp.x - base.x).abs() < revealable.half_width_px;

        match (*state, behind) {
            (RevealState::Full, true) => {
                *state = RevealState::Revealing(0.0);
            }
            (RevealState::Hiding(progress), true) => {
                // Reverse: convert hiding progress to revealing progress.
                *state = RevealState::Revealing(1.0 - progress);
            }
            (RevealState::Revealed, false) => {
                *state = RevealState::Hiding(0.0);
            }
            (RevealState::Revealing(progress), false) => {
                // Reverse: convert revealing progress to hiding progress.
                *state = RevealState::Hiding(1.0 - progress);
            }
            _ => {}
        }
    }
}

/// Animate the crossfade between full and stump sprites.
pub fn animate_reveals(
    time: Res<Time>,
    mut revealables: Query<(&mut RevealState, &Children)>,
    mut sprites: Query<&mut Sprite>,
    full_q: Query<(), With<FullSprite>>,
    stump_q: Query<(), With<StumpSprite>>,
) {
    let dt = time.delta_secs();

    for (mut state, children) in &mut revealables {
        let (progress, revealing) = match *state {
            RevealState::Revealing(p) => (p, true),
            RevealState::Hiding(p) => (p, false),
            _ => continue,
        };

        let new_progress = (progress + dt / REVEAL_DURATION_SECS).min(1.0);

        // Compute alpha values.
        // Revealing: full goes from 1->0, stump goes from 0->1.
        // Hiding: full goes from 0->1, stump goes from 1->0.
        let (full_alpha, stump_alpha) = if revealing {
            (1.0 - new_progress, new_progress)
        } else {
            (new_progress, 1.0 - new_progress)
        };

        for child in children.iter() {
            if let Ok(mut sprite) = sprites.get_mut(*child) {
                if full_q.contains(*child) {
                    sprite.color = sprite.color.with_alpha(full_alpha);
                } else if stump_q.contains(*child) {
                    sprite.color = sprite.color.with_alpha(stump_alpha);
                }
            }
        }

        if new_progress >= 1.0 {
            *state = if revealing {
                RevealState::Revealed
            } else {
                RevealState::Full
            };
        } else {
            *state = if revealing {
                RevealState::Revealing(new_progress)
            } else {
                RevealState::Hiding(new_progress)
            };
        }
    }
}
```

- [ ] **Step 2: Register module in `level/src/lib.rs`**

Add: `pub mod reveal;`

- [ ] **Step 3: Register systems in `level/src/plugin.rs`**

Add both systems to the `Update` set running in `GameState::Playing`:

```rust
use crate::reveal;

// In Update systems:
reveal::detect_reveals,
reveal::animate_reveals,
```

- [ ] **Step 4: Verify build**

Run: `cargo build`

- [ ] **Step 5: Commit**

```bash
git add level/src/reveal.rs level/src/lib.rs level/src/plugin.rs
git commit -m "feat: add stump reveal crossfade system for trees and decorations"
```

---

### Task 9: Add Reveal to Large Decorations

**Files:**
- Modify: `level/src/decorations.rs`

Large/medium decorations (height >= 24px) should also participate in the reveal system. Since they don't have stump sprites, they fade to alpha 0.3 instead.

- [ ] **Step 1: Update DecorationDef to track reveal eligibility**

```rust
struct DecorationDef {
    path: &'static str,
    width_px: f32,
    height_px: f32,
    rustleable: bool,
    revealable: bool,  // true for height_px >= 24.0
}
```

Update all decoration pool entries -- set `revealable: true` for items with `height_px >= 24.0`:
- Darkwood: thorn_bush (24x24), spider_web (32x16 -- 16 tall, no), dead_branch (24x16 -- 16 tall, no)
- Greenwood: berry_bush (24x24), fern (24x24), mossy_rock (24x24), fallen_log (24x16 -- no)
- City: wooden_crate (24x24), barrel (24x24), hay_bale (24x24), cart (32x16 -- no)

- [ ] **Step 2: Update `spawn_decoration` for revealable decorations**

For revealable decorations, use the same parent+child pattern as trees but with alpha fade instead of stump swap:

```rust
use models::reveal::{FullSprite, Revealable, RevealState, StumpSprite};

fn spawn_decoration(
    commands: &mut Commands,
    asset_server: &AssetServer,
    def: &DecorationDef,
    world_x: f32,
    world_y: f32,
) {
    let z = Layer::World.z_f32() - world_y * Y_SORT_SCALE;

    if def.revealable {
        let parent = commands
            .spawn((
                Decoration,
                Revealable {
                    canopy_height_px: def.height_px,
                    half_width_px: def.width_px / 2.0,
                },
                RevealState::default(),
                Transform::from_xyz(world_x, world_y, z),
                Visibility::default(),
            ))
            .id();

        let full_child = commands
            .spawn((
                FullSprite,
                Sprite {
                    image: asset_server.load(def.path),
                    custom_size: Some(Vec2::new(def.width_px, def.height_px)),
                    ..default()
                },
                Transform::IDENTITY,
            ))
            .id();

        // "Stump" for decorations is the same sprite at low alpha.
        let stump_child = commands
            .spawn((
                StumpSprite,
                Sprite {
                    image: asset_server.load(def.path),
                    custom_size: Some(Vec2::new(def.width_px, def.height_px)),
                    color: palette::TRANSPARENT,
                    ..default()
                },
                Transform::IDENTITY,
            ))
            .id();

        commands.entity(parent).add_children(&[full_child, stump_child]);

        if def.rustleable {
            commands.entity(parent).insert(Rustleable);
        }
    } else {
        // Small decorations: simple sprite, no reveal.
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
}
```

Wait -- for decorations the "stump" should fade to 0.3 alpha, not swap sprites. The reveal system in Task 8 swaps between full (alpha 1->0) and stump (alpha 0->1). For decorations, the stump child IS the same sprite, so when revealed the full is invisible and the stump shows at full alpha -- that's not right.

Instead, for decorations without true stumps, the stump alpha target should be 0.3 rather than 1.0. Update the reveal system to check: if the StumpSprite has the same image as FullSprite, cap stump_alpha at 0.3. Or simpler: add a field to `Revealable`:

```rust
// In models/src/reveal.rs, add:
pub struct Revealable {
    pub canopy_height_px: f32,
    pub half_width_px: f32,
    /// Minimum alpha when revealed (1.0 = full swap to stump, 0.3 = semi-transparent).
    pub revealed_full_alpha: f32,
}
```

For trees: `revealed_full_alpha: 0.0` (full sprite disappears, stump appears).
For decorations: `revealed_full_alpha: 0.3` (full sprite fades to 30%, no stump child needed).

This means decorations don't need a StumpSprite child at all -- simplify:

```rust
fn spawn_decoration(
    commands: &mut Commands,
    asset_server: &AssetServer,
    def: &DecorationDef,
    world_x: f32,
    world_y: f32,
) {
    let z = Layer::World.z_f32() - world_y * Y_SORT_SCALE;

    let mut entity = commands.spawn((
        Decoration,
        Sprite {
            image: asset_server.load(def.path),
            custom_size: Some(Vec2::new(def.width_px, def.height_px)),
            ..default()
        },
        Transform::from_xyz(world_x, world_y, z),
    ));

    if def.revealable {
        entity.insert((
            Revealable {
                canopy_height_px: def.height_px,
                half_width_px: def.width_px / 2.0,
                revealed_full_alpha: 0.3,
            },
            RevealState::default(),
        ));
    }

    if def.rustleable {
        entity.insert(Rustleable);
    }
}
```

- [ ] **Step 3: Update reveal system to handle `revealed_full_alpha`**

In `level/src/reveal.rs`, update `animate_reveals` to use `revealable.revealed_full_alpha`:

For entities with children (trees): full alpha goes from 1.0 -> 0.0, stump from 0.0 -> 1.0.
For entities without children (decorations): sprite alpha goes from 1.0 -> `revealed_full_alpha`.

Add a separate simpler system for childless revealables:

```rust
/// Animate alpha fade for revealable entities without child sprites (decorations).
pub fn animate_reveals_simple(
    time: Res<Time>,
    mut query: Query<(&Revealable, &mut RevealState, &mut Sprite), Without<Children>>,
) {
    let dt = time.delta_secs();

    for (revealable, mut state, mut sprite) in &mut query {
        let (progress, revealing) = match *state {
            RevealState::Revealing(p) => (p, true),
            RevealState::Hiding(p) => (p, false),
            _ => continue,
        };

        let new_progress = (progress + dt / REVEAL_DURATION_SECS).min(1.0);
        let target = revealable.revealed_full_alpha;

        let alpha = if revealing {
            1.0 - (1.0 - target) * new_progress
        } else {
            target + (1.0 - target) * new_progress
        };

        sprite.color = sprite.color.with_alpha(alpha);

        if new_progress >= 1.0 {
            *state = if revealing { RevealState::Revealed } else { RevealState::Full };
        } else {
            *state = if revealing { RevealState::Revealing(new_progress) } else { RevealState::Hiding(new_progress) };
        }
    }
}
```

Register this system in plugin.rs alongside the existing reveal systems.

- [ ] **Step 4: Update `Revealable` in models**

Add `revealed_full_alpha` field to `Revealable` in `models/src/reveal.rs`. Update tree spawning in scenery.rs to set `revealed_full_alpha: 0.0`.

- [ ] **Step 5: Verify build**

Run: `cargo build`

- [ ] **Step 6: Commit**

```bash
git add level/src/decorations.rs level/src/reveal.rs level/src/plugin.rs models/src/reveal.rs
git commit -m "feat: add reveal fade to large decorations when player walks behind"
```

---

### Task 10: Generate Tree Assets with PixelLab

**Files:**
- Create: 16 tree sprites in `assets/sprites/scenery/trees/{city,greenwood,darkwood}/`
- Delete: `assets/sprites/scenery/trees/tree_oak.webp`, `tree_pine.webp`

- [ ] **Step 1: Create directories**

```bash
mkdir -p assets/sprites/scenery/trees/{city,greenwood,darkwood}
```

- [ ] **Step 2: Generate full trees (batch of 4, wait 60s, repeat)**

Use `mcp__pixellab__create_map_object` with `width: 48, height: 64` for each. Style: `view: "low top-down"`, `outline: "single color outline"`, `shading: "basic shading"`, `detail: "medium detail"`.

Batch 1 (4 jobs): city_ornamental, city_fruit, green_oak, green_birch
Batch 2 (4 jobs): green_maple, dark_gnarled, dark_dead, dark_willow

- [ ] **Step 3: Generate stumps (batch of 4, wait 60s, repeat)**

Use `width: 48, height: 24` for each. Same style params.

Batch 3: city_ornamental_stump, city_fruit_stump, green_oak_stump, green_birch_stump
Batch 4: green_maple_stump, dark_gnarled_stump, dark_dead_stump, dark_willow_stump

- [ ] **Step 4: Download and convert all to webp**

```bash
nix-shell -p imagemagick --run 'for f in assets/sprites/scenery/trees/**/*.png; do magick "$f" -quality 90 "${f%.png}.webp" && rm "$f"; done'
```

- [ ] **Step 5: Delete old tree assets**

```bash
git rm assets/sprites/scenery/trees/tree_oak.webp assets/sprites/scenery/trees/tree_pine.webp
```

- [ ] **Step 6: Commit**

```bash
git add assets/sprites/scenery/trees/
git commit -m "feat: add 16 biome-specific tree sprites (8 full + 8 stump)"
```

---

### Task 11: Verify End-to-End

- [ ] **Step 1: Build and run**

```bash
cargo build && trunk serve
```

Verify:
1. Trees are 48x64, dense, with biome-specific variants
2. Player, NPCs, trees, and decorations y-sort correctly
3. Walking behind a tree crossfades to stump view
4. Walking behind large decorations fades them to semi-transparent
5. Biome borders blend -- greenwood near darkwood shows some dark trees/decorations
6. Tileset matches the dominant biome of each area
7. No mixels -- all sprites are crisp at native resolution

- [ ] **Step 2: Run lints**

```bash
cargo clippy && cargo fmt -- --check
```

- [ ] **Step 3: Final commit if needed**

```bash
git add -u
git commit -m "fix: address lint issues from tree/reveal system"
```
