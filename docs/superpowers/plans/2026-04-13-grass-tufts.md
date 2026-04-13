# Grass Tufts Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add small decorative grass sprites that spawn in patches on grass tiles, with a wind-reactive sway animation driven by the weather system's `WindStrength` resource.

**Architecture:** A `GrassTuft` marker and `WindSway` component live in models. Spawning follows the same deterministic-seed, biome-pool, blended-alignment pattern as decorations but with a unique salt to avoid overlap. A sway animation system reads `Res<WindStrength>` and applies a sinusoidal rotation to each tuft. 20-30 tufts per area, spawned/despawned with area lifecycle.

**Tech Stack:** Rust, Bevy 0.18, PixelLab (grass sprites)

---

## File Structure

| Action | File | Responsibility |
|--------|------|----------------|
| Create | `models/src/grass.rs` | `GrassTuft` marker, `WindSway` component |
| Modify | `models/src/lib.rs` | Add `pub mod grass` |
| Create | `level/src/grass.rs` | Spawning logic, sway animation system, despawn |
| Modify | `level/src/lib.rs` | Add `pub mod grass` |
| Modify | `level/src/spawning.rs` | Call `grass::spawn_area_grass` in `ensure_area_spawned` |
| Modify | `level/src/plugin.rs` | Register sway system, despawn system |

---

## Task 1: Add Grass Model Components

**Files:**
- Create: `models/src/grass.rs`
- Modify: `models/src/lib.rs`

- [ ] **Step 1: Create the grass module**

```rust
// models/src/grass.rs

use bevy::prelude::Component;

/// Marker for decorative grass tuft entities.
#[derive(Component, Default)]
pub struct GrassTuft;

/// Per-entity phase offset for wind sway animation.
///
/// The sway system applies:
/// `rotation = sin(time * FREQUENCY + phase) * MAX_ANGLE * wind_strength`
#[derive(Component)]
pub struct WindSway {
    /// Random phase offset in radians, set at spawn time.
    pub phase: f32,
}
```

- [ ] **Step 2: Register in models/src/lib.rs**

Add in alphabetical order (after `game_states`):

```rust
pub mod grass;
```

- [ ] **Step 3: Commit**

```bash
git add models/src/grass.rs models/src/lib.rs
git commit -m "Add GrassTuft marker and WindSway component"
```

---

## Task 2: Implement Grass Spawning and Sway

**Files:**
- Create: `level/src/grass.rs`

- [ ] **Step 1: Write spawning and animation systems**

```rust
// level/src/grass.rs

use std::f32::consts::TAU;

use bevy::math::IVec2;
use bevy::prelude::*;
use models::decoration::Biome;
use models::grass::{GrassTuft, WindSway};
use models::layer::Layer;
use models::wind::WindStrength;

use crate::area::{Area, MAP_HEIGHT, MAP_WIDTH};
use crate::blending;
use crate::spawning::{area_world_offset, TILE_SIZE_PX};
use crate::terrain::{tile_hash, Terrain};
use crate::world::WorldMap;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

#[allow(clippy::as_conversions)]
const MAP_W_PX: f32 = MAP_WIDTH as f32 * TILE_SIZE_PX as f32;
#[allow(clippy::as_conversions)]
const MAP_H_PX: f32 = MAP_HEIGHT as f32 * TILE_SIZE_PX as f32;

/// Y-sort scale factor for z-ordering within the World layer.
const Y_SORT_SCALE: f32 = 0.001;

/// Minimum grass tufts per area.
const MIN_GRASS_PER_AREA: usize = 20;
/// Maximum grass tufts per area.
const MAX_GRASS_PER_AREA: usize = 30;

/// Inset from area edges (same as decorations).
const EDGE_INSET: u16 = 2;

/// Unique salt added to area seed to avoid overlapping decoration/tree positions.
const GRASS_SEED_SALT: u32 = 55_555;

/// Sway oscillation frequency in Hz.
const SWAY_FREQUENCY_HZ: f32 = 3.0;
/// Maximum sway angle in radians.
const SWAY_MAX_ANGLE_RAD: f32 = 0.1;

// ---------------------------------------------------------------------------
// Grass sprite definitions
// ---------------------------------------------------------------------------

struct GrassDef {
    path: &'static str,
}

const CITY_GRASS: &[GrassDef] = &[
    GrassDef {
        path: "sprites/scenery/grass/city/grass_small.webp",
    },
    GrassDef {
        path: "sprites/scenery/grass/city/grass_medium.webp",
    },
    GrassDef {
        path: "sprites/scenery/grass/city/grass_large.webp",
    },
];

const GREENWOOD_GRASS: &[GrassDef] = &[
    GrassDef {
        path: "sprites/scenery/grass/greenwood/grass_small.webp",
    },
    GrassDef {
        path: "sprites/scenery/grass/greenwood/grass_medium.webp",
    },
    GrassDef {
        path: "sprites/scenery/grass/greenwood/grass_large.webp",
    },
];

const DARKWOOD_GRASS: &[GrassDef] = &[
    GrassDef {
        path: "sprites/scenery/grass/darkwood/grass_small.webp",
    },
    GrassDef {
        path: "sprites/scenery/grass/darkwood/grass_medium.webp",
    },
    GrassDef {
        path: "sprites/scenery/grass/darkwood/grass_large.webp",
    },
];

// ---------------------------------------------------------------------------
// Spawning
// ---------------------------------------------------------------------------

/// Spawn 20-30 grass tufts for a single area on grass tiles.
pub fn spawn_area_grass(
    commands: &mut Commands,
    asset_server: &AssetServer,
    area: &Area,
    area_pos: IVec2,
    world: &WorldMap,
) {
    let tile_px = f32::from(TILE_SIZE_PX);
    let base = area_world_offset(area_pos);
    let base_offset_x = base.x - MAP_W_PX / 2.0;
    let base_offset_y = base.y - MAP_H_PX / 2.0;

    let ax = u32::from_ne_bytes(area_pos.x.to_ne_bytes());
    let ay = u32::from_ne_bytes(area_pos.y.to_ne_bytes());
    let area_seed = ax
        .wrapping_mul(2_654_435_761)
        .wrapping_add(ay.wrapping_mul(1_013_904_223));

    let grass_seed = area_seed.wrapping_add(GRASS_SEED_SALT);

    // Collect candidate grass tiles (inset from edges).
    let mut candidates: Vec<(u32, u32)> = Vec::new();
    for x in EDGE_INSET..(MAP_WIDTH - EDGE_INSET) {
        for y in EDGE_INSET..(MAP_HEIGHT - EDGE_INSET) {
            let xu = u32::from(x);
            let yu = u32::from(y);
            if area.terrain_at(xu, yu) == Some(Terrain::Grass) {
                candidates.push((xu, yu));
            }
        }
    }

    if candidates.is_empty() {
        return;
    }

    // Deterministically shuffle candidates.
    let len = candidates.len();
    let mut rng = u64::from(grass_seed);
    for i in (1..len).rev() {
        rng = lcg(rng);
        #[allow(clippy::as_conversions)]
        let j = (rng % u64::try_from(i + 1).expect("i+1 fits u64")) as usize;
        candidates.swap(i, j);
    }

    // Pick tuft count (20-30) deterministically.
    rng = lcg(rng);
    #[allow(clippy::as_conversions)]
    let range = (MAX_GRASS_PER_AREA - MIN_GRASS_PER_AREA + 1) as u64;
    #[allow(clippy::as_conversions)]
    let count = MIN_GRASS_PER_AREA + (rng % range) as usize;
    let count = count.min(candidates.len());

    for (i, &(xu, yu)) in candidates.iter().take(count).enumerate() {
        let effective_alignment =
            blending::blended_alignment(area.alignment, xu, yu, area_pos, world);
        let biome = Biome::from_alignment(effective_alignment);
        let pool = match biome {
            Biome::City => CITY_GRASS,
            Biome::Greenwood => GREENWOOD_GRASS,
            Biome::Darkwood => DARKWOOD_GRASS,
        };

        let variant = tile_hash(
            xu,
            yu,
            grass_seed.wrapping_add(u32::try_from(i).expect("i fits u32")),
        ) % pool.len();
        let def = &pool[variant];

        let world_x = base_offset_x
            + f32::from(u16::try_from(xu).expect("xu fits u16")) * tile_px
            + tile_px / 2.0;
        let world_y = base_offset_y
            + f32::from(u16::try_from(yu).expect("yu fits u16")) * tile_px
            + tile_px / 2.0;

        let z = Layer::World.z_f32() - world_y * Y_SORT_SCALE;

        // Derive phase from tile hash so each tuft sways independently.
        #[allow(clippy::as_conversions)]
        let phase = (tile_hash(xu, yu, grass_seed) % 6283) as f32 / 1000.0;

        commands.spawn((
            GrassTuft,
            WindSway { phase },
            Sprite {
                image: asset_server.load(def.path),
                ..default()
            },
            Transform::from_xyz(world_x, world_y, z),
        ));
    }
}

// ---------------------------------------------------------------------------
// Animation
// ---------------------------------------------------------------------------

/// Oscillate grass tufts based on wind strength and per-entity phase.
pub fn animate_grass_sway(
    time: Res<Time>,
    wind: Res<WindStrength>,
    mut query: Query<(&WindSway, &mut Transform), With<GrassTuft>>,
) {
    let elapsed = time.elapsed_secs();
    for (sway, mut tf) in &mut query {
        let angle =
            (elapsed * SWAY_FREQUENCY_HZ + sway.phase).sin() * SWAY_MAX_ANGLE_RAD * wind.0;
        tf.rotation = Quat::from_rotation_z(angle);
    }
}

// ---------------------------------------------------------------------------
// Despawn
// ---------------------------------------------------------------------------

/// Despawn all grass tufts on game exit.
pub fn despawn_grass(mut commands: Commands, query: Query<Entity, With<GrassTuft>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

// ---------------------------------------------------------------------------
// LCG
// ---------------------------------------------------------------------------

fn lcg(state: u64) -> u64 {
    state
        .wrapping_mul(6_364_136_223_846_793_005)
        .wrapping_add(1_442_695_040_888_963_407)
}
```

- [ ] **Step 2: Commit**

```bash
git add level/src/grass.rs
git commit -m "Implement grass tuft spawning and wind sway animation"
```

---

## Task 3: Wire Grass Into Level Systems

**Files:**
- Modify: `level/src/lib.rs`
- Modify: `level/src/spawning.rs`
- Modify: `level/src/plugin.rs`

- [ ] **Step 1: Add grass module to level/src/lib.rs**

Add in alphabetical order (after `galen`):

```rust
pub mod grass;
```

- [ ] **Step 2: Call spawn_area_grass in spawning.rs**

In `level/src/spawning.rs`, add the grass import at the top:

```rust
use crate::grass;
```

Then in the `ensure_area_spawned` function, add the grass spawn call after `npcs::spawn_npc_for_area`:

```rust
fn ensure_area_spawned(
    commands: &mut Commands,
    asset_server: &AssetServer,
    atlas_layouts: &mut Assets<TextureAtlasLayout>,
    world: &WorldMap,
    area_pos: IVec2,
    spawned: &mut SpawnedAreas,
) {
    if spawned.0.contains(&area_pos) {
        return;
    }
    let dense_forest = Area::dense_forest();
    let area = world.get_area(area_pos).unwrap_or(&dense_forest);
    spawn_area_tilemap(commands, asset_server, world, area, area_pos);
    scenery::spawn_area_scenery_at(commands, asset_server, area, area_pos, world);
    decorations::spawn_area_decorations(commands, asset_server, area, area_pos, world);
    npcs::spawn_npc_for_area(commands, asset_server, atlas_layouts, area, area_pos);
    grass::spawn_area_grass(commands, asset_server, area, area_pos, world);
    spawned.0.insert(area_pos);
}
```

- [ ] **Step 3: Register grass systems in level/src/plugin.rs**

Add `use crate::grass;` to the imports.

Add `grass::animate_grass_sway` to the `Update` system set (inside `run_if(in_state(GameState::Playing))`):

```rust
grass::animate_grass_sway,
```

Add `grass::despawn_grass` to the `OnExit(GameState::Playing)` system set (inside `run_if(should_despawn_world)`):

```rust
grass::despawn_grass,
```

The full plugin.rs after this change:

```rust
use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::TilemapPlugin;
use models::alignment::PlayerAlignment;
use models::game_states::{should_despawn_world, GameState};
use models::weather::WeatherState;
use models::wind::WindStrength;

use crate::bark_bubbles;
use crate::decorations;
use crate::exit;
use crate::galen;
use crate::grass;
use crate::npc_anim;
use crate::npc_labels::{self, InteractIconState};
use crate::npc_wander;
use crate::npcs;
use crate::reveal;
use crate::scenery;
use crate::spawning::{self, SpawnedAreas};
use crate::weather;
use crate::world::{AreaChanged, WorldMap};

pub use crate::area::{MAP_HEIGHT, MAP_WIDTH};
pub use crate::spawning::{tile_size, TILE_SIZE_PX};

pub struct LevelPlugin;

impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<InteractIconState>()
            .init_resource::<SpawnedAreas>()
            .init_resource::<WindStrength>()
            .init_resource::<WeatherState>()
            .add_plugins(TilemapPlugin)
            .add_message::<AreaChanged>()
            .insert_resource(WorldMap::new(rand::random(), 50))
            .add_systems(
                OnEnter(GameState::Playing),
                (
                    regenerate_world,
                    spawning::spawn_initial_areas,
                    galen::spawn_galen,
                    exit::spawn_exit,
                )
                    .chain(),
            )
            .add_systems(
                Update,
                (
                    spawning::ensure_neighbors_on_area_change,
                    scenery::animate_rustle,
                    npc_labels::attach_labels,
                    npc_labels::sync_interact_icon,
                    npc_anim::advance_npc_frame,
                    npc_anim::reset_npc_anim_on_change,
                    npc_wander::wander_npcs,
                    npcs::update_npc_z,
                    galen::update_galen_z,
                    bark_bubbles::spawn_bark_bubble,
                    bark_bubbles::tick_bark_bubbles,
                    reveal::detect_reveals,
                    reveal::animate_reveals,
                    weather::weather_state_machine,
                    weather::sync_wind_strength,
                    weather::spawn_weather_particles,
                    weather::update_weather_particles,
                    grass::animate_grass_sway,
                )
                    .run_if(in_state(GameState::Playing)),
            )
            .add_systems(
                OnExit(GameState::Playing),
                (
                    spawning::despawn_all_areas,
                    scenery::despawn_scenery,
                    decorations::despawn_decorations,
                    npcs::despawn_npcs,
                    galen::despawn_galen,
                    exit::despawn_exit,
                    weather::despawn_weather_particles,
                    grass::despawn_grass,
                )
                    .run_if(should_despawn_world),
            );
    }
}

/// Regenerate the world with a fresh seed, biased toward the player's
/// dominant faction alignment.  Skips if a world is already loaded
/// (e.g. returning from Dialogue or Paused).
fn regenerate_world(
    mut world: ResMut<WorldMap>,
    alignment: Res<PlayerAlignment>,
    spawned: Res<SpawnedAreas>,
) {
    if !spawned.0.is_empty() {
        return;
    }
    let dominant = alignment.dominant_area_alignment();
    *world = WorldMap::new(rand::random(), dominant);
}
```

- [ ] **Step 4: Verify build**

```bash
cargo build
```

Expected: compiles with no errors.

- [ ] **Step 5: Commit**

```bash
git add level/src/lib.rs level/src/spawning.rs level/src/plugin.rs
git commit -m "Wire grass tufts into area spawning and level plugin"
```

---

## Task 4: Generate Grass Sprites via PixelLab

**Files:**
- Create: 9 sprites in `assets/sprites/scenery/grass/`

All sprites use the `create_map_object` PixelLab tool. Style suffix for all: "Warm earthy palette with hue-shifted shadows toward cool purple and highlights toward warm gold. Moderate saturation, clean readable forms, storybook fantasy RPG style. 16-bit pixel art aesthetic."

- [ ] **Step 1: Generate city/grass_small.webp**

Use `mcp__pixellab__create_map_object` with:
- `description`: "A small single tuft of short, trimmed yellow-green grass, 2-3 blades, manicured lawn style. Warm earthy palette with hue-shifted shadows toward cool purple and highlights toward warm gold. Moderate saturation, clean readable forms, storybook fantasy RPG style. 16-bit pixel art aesthetic."
- `view`: "low top-down"
- `outline`: "single color outline"
- `shading`: "basic shading"
- `detail`: "medium detail"
- `width`: 8
- `height`: 8
- `padding`: 4

Save to `assets/sprites/scenery/grass/city/grass_small.webp`.

- [ ] **Step 2: Generate city/grass_medium.webp**

Same style but:
- `description`: "A wider clump of short, trimmed yellow-green grass, manicured lawn style, several blades. Warm earthy palette with hue-shifted shadows toward cool purple and highlights toward warm gold. Moderate saturation, clean readable forms, storybook fantasy RPG style. 16-bit pixel art aesthetic."
- `width`: 16
- `height`: 8
- `padding`: 8

Save to `assets/sprites/scenery/grass/city/grass_medium.webp`.

- [ ] **Step 3: Generate city/grass_large.webp**

Same style but:
- `description`: "A dense patch of short, trimmed yellow-green grass, manicured lawn, multiple clumps. Warm earthy palette with hue-shifted shadows toward cool purple and highlights toward warm gold. Moderate saturation, clean readable forms, storybook fantasy RPG style. 16-bit pixel art aesthetic."
- `width`: 16
- `height`: 16
- `padding`: 8

Save to `assets/sprites/scenery/grass/city/grass_large.webp`.

- [ ] **Step 4: Generate greenwood/grass_small.webp**

- `description`: "A small single tuft of lush, vibrant rich green grass, 2-3 tall varied blades. Warm earthy palette with hue-shifted shadows toward cool purple and highlights toward warm gold. Moderate saturation, clean readable forms, storybook fantasy RPG style. 16-bit pixel art aesthetic."
- `width`: 8
- `height`: 8
- `padding`: 4

Save to `assets/sprites/scenery/grass/greenwood/grass_small.webp`.

- [ ] **Step 5: Generate greenwood/grass_medium.webp**

- `description`: "A wider clump of lush, vibrant rich green grass, varied heights, several tall blades. Warm earthy palette with hue-shifted shadows toward cool purple and highlights toward warm gold. Moderate saturation, clean readable forms, storybook fantasy RPG style. 16-bit pixel art aesthetic."
- `width`: 16
- `height`: 8
- `padding`: 8

Save to `assets/sprites/scenery/grass/greenwood/grass_medium.webp`.

- [ ] **Step 6: Generate greenwood/grass_large.webp**

- `description`: "A dense patch of lush, vibrant rich green grass, tall varied blades, fairy-tale forest floor. Warm earthy palette with hue-shifted shadows toward cool purple and highlights toward warm gold. Moderate saturation, clean readable forms, storybook fantasy RPG style. 16-bit pixel art aesthetic."
- `width`: 16
- `height`: 16
- `padding`: 8

Save to `assets/sprites/scenery/grass/greenwood/grass_large.webp`.

- [ ] **Step 7: Generate darkwood/grass_small.webp**

- `description`: "A small single tuft of sparse, wilted gray-brown dead grass, 2-3 dry bent blades. Warm earthy palette with hue-shifted shadows toward cool purple and highlights toward warm gold. Moderate saturation, clean readable forms, storybook fantasy RPG style. 16-bit pixel art aesthetic."
- `width`: 8
- `height`: 8
- `padding`: 4

Save to `assets/sprites/scenery/grass/darkwood/grass_small.webp`.

- [ ] **Step 8: Generate darkwood/grass_medium.webp**

- `description`: "A wider clump of sparse, wilted gray-brown dead grass, dry bent blades, decayed look. Warm earthy palette with hue-shifted shadows toward cool purple and highlights toward warm gold. Moderate saturation, clean readable forms, storybook fantasy RPG style. 16-bit pixel art aesthetic."
- `width`: 16
- `height`: 8
- `padding`: 8

Save to `assets/sprites/scenery/grass/darkwood/grass_medium.webp`.

- [ ] **Step 9: Generate darkwood/grass_large.webp**

- `description`: "A dense patch of sparse, wilted gray-brown dead grass, dry bent blades, oppressive decayed forest floor. Warm earthy palette with hue-shifted shadows toward cool purple and highlights toward warm gold. Moderate saturation, clean readable forms, storybook fantasy RPG style. 16-bit pixel art aesthetic."
- `width`: 16
- `height`: 16
- `padding`: 8

Save to `assets/sprites/scenery/grass/darkwood/grass_large.webp`.

- [ ] **Step 10: Commit**

```bash
git add assets/sprites/scenery/grass/
git commit -m "Add grass tuft sprites for all three biomes (9 sprites)"
```

---

## Task 5: Verify Build

- [ ] **Step 1: Full build check**

```bash
cargo build
```

Expected: compiles with no errors.

- [ ] **Step 2: Run clippy**

```bash
cargo clippy
```

Expected: no warnings or errors.
