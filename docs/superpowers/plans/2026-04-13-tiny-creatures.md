# Tiny Creatures Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add ambient wildlife (mice, butterflies, crows, etc.) that spawns per-area, wanders randomly, and flees from the player. Pure visual -- no collision, no gameplay impact.

**Architecture:** A `CreatureAi` component drives a three-state machine (Idle/Wander/Flee) with timer-based transitions and player-distance-triggered flee. Each creature type has a `MovementType` (Ground or Flying) that determines sprite flipping and vertical bobbing behavior. Spawning follows the same deterministic-seed, biome-pool, blended-alignment pattern as decorations and grass. 4-8 creatures per area.

**Tech Stack:** Rust, Bevy 0.18, PixelLab (creature sprite sheets)

---

## File Structure

| Action | File | Responsibility |
|--------|------|----------------|
| Create | `models/src/creature.rs` | `Creature` marker, `CreatureAi`, `CreatureState`, `MovementType` |
| Modify | `models/src/lib.rs` | Add `pub mod creature` |
| Create | `level/src/creatures.rs` | Biome pools, spawning, AI systems (state transitions, movement, animation) |
| Modify | `level/src/lib.rs` | Add `pub mod creatures` |
| Modify | `level/src/spawning.rs` | Call `creatures::spawn_area_creatures` in `ensure_area_spawned` |
| Modify | `level/src/plugin.rs` | Register creature systems, despawn |

---

## Task 1: Add Creature Model Components

**Files:**
- Create: `models/src/creature.rs`
- Modify: `models/src/lib.rs`

- [ ] **Step 1: Create the creature module**

```rust
// models/src/creature.rs

use bevy::math::Vec2;
use bevy::prelude::{Component, Timer, TimerMode};

/// Marker for ambient creature entities.
#[derive(Component, Default)]
pub struct Creature;

/// Movement behavior category for a creature.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MovementType {
    /// Moves along the ground, flips sprite horizontally to face direction.
    Ground,
    /// Moves freely with slight vertical bobbing.
    Flying,
}

/// AI state for a creature entity.
#[derive(Debug, Clone)]
pub enum CreatureState {
    /// Stationary, showing idle frame.
    Idle,
    /// Moving in a random direction.
    Wander(Vec2),
    /// Fleeing away from the player.
    Flee,
}

/// Minimum idle duration in seconds.
const MIN_IDLE_SECS: f32 = 2.0;
/// Maximum idle duration in seconds.
const MAX_IDLE_SECS: f32 = 5.0;
/// Minimum wander duration in seconds.
const MIN_WANDER_SECS: f32 = 1.0;
/// Maximum wander duration in seconds.
const MAX_WANDER_SECS: f32 = 3.0;

impl CreatureState {
    /// Create a new Idle state with a random timer duration.
    pub fn new_idle(seed: u32) -> (Self, Timer) {
        let duration = seeded_range(seed, MIN_IDLE_SECS, MAX_IDLE_SECS);
        (Self::Idle, Timer::from_seconds(duration, TimerMode::Once))
    }

    /// Create a new Wander state with a random direction and timer duration.
    pub fn new_wander(seed: u32) -> (Self, Timer) {
        let angle = seeded_frac(seed) * std::f32::consts::TAU;
        let direction = Vec2::new(angle.cos(), angle.sin());
        let duration =
            seeded_range(seed.wrapping_add(1), MIN_WANDER_SECS, MAX_WANDER_SECS);
        (
            Self::Wander(direction),
            Timer::from_seconds(duration, TimerMode::Once),
        )
    }
}

/// AI component driving creature behavior.
#[derive(Component)]
pub struct CreatureAi {
    /// Current behavioral state.
    pub state: CreatureState,
    /// Timer for state duration (Idle/Wander).
    pub timer: Timer,
    /// Movement speed in pixels per second.
    pub speed: f32,
    /// Whether this creature walks or flies.
    pub movement: MovementType,
    /// Monotonically increasing seed for random decisions.
    pub seed_counter: u32,
}

impl CreatureAi {
    /// Create a new AI in Idle state.
    pub fn new(speed: f32, movement: MovementType, seed: u32) -> Self {
        let (state, timer) = CreatureState::new_idle(seed);
        Self {
            state,
            timer,
            speed,
            movement,
            seed_counter: seed,
        }
    }

    /// Advance the seed counter and return a fresh seed.
    pub fn next_seed(&mut self) -> u32 {
        self.seed_counter = self.seed_counter.wrapping_add(1);
        self.seed_counter
    }
}

/// Hash a seed to a fraction in [0.0, 1.0).
fn seeded_frac(seed: u32) -> f32 {
    let h = seed
        .wrapping_mul(374_761_393)
        .wrapping_add(668_265_263);
    let h = (h ^ (h >> 13)).wrapping_mul(1_274_126_177);
    let h = h ^ (h >> 16);
    #[allow(clippy::as_conversions)]
    let frac = (h % 10000) as f32 / 10000.0;
    frac
}

/// Generate a random f32 in [min, max) from a seed.
fn seeded_range(seed: u32, min: f32, max: f32) -> f32 {
    min + seeded_frac(seed) * (max - min)
}
```

- [ ] **Step 2: Register in models/src/lib.rs**

Add in alphabetical order (after `attack`):

```rust
pub mod creature;
```

- [ ] **Step 3: Commit**

```bash
git add models/src/creature.rs models/src/lib.rs
git commit -m "Add Creature marker, CreatureAi, and CreatureState model"
```

---

## Task 2: Implement Creature Spawning and AI

**Files:**
- Create: `level/src/creatures.rs`

- [ ] **Step 1: Write creature pools, spawning, and AI systems**

```rust
// level/src/creatures.rs

use bevy::math::IVec2;
use bevy::prelude::*;
use models::creature::{Creature, CreatureAi, CreatureState, MovementType};
use models::decoration::Biome;
use models::layer::Layer;
use models::player::Player;

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

/// Minimum creatures per area.
const MIN_CREATURES_PER_AREA: usize = 4;
/// Maximum creatures per area.
const MAX_CREATURES_PER_AREA: usize = 8;

/// Inset from area edges (same as decorations).
const EDGE_INSET: u16 = 2;

/// Unique salt to avoid overlapping other spawn positions.
const CREATURE_SEED_SALT: u32 = 77_777;

/// Wander speed in pixels per second (1-2 tiles/sec, each tile = 16px).
const WANDER_SPEED_PX: f32 = 24.0;
/// Flee speed in pixels per second (4 tiles/sec).
const FLEE_SPEED_PX: f32 = 64.0;

/// Distance in pixels at which creatures start fleeing (3 tiles).
const FLEE_TRIGGER_DISTANCE_PX: f32 = 48.0;
/// Distance in pixels at which creatures stop fleeing (5 tiles).
const FLEE_STOP_DISTANCE_PX: f32 = 80.0;

/// Vertical bobbing amplitude for flying creatures (pixels).
const FLY_BOB_AMPLITUDE_PX: f32 = 2.0;
/// Vertical bobbing frequency for flying creatures (Hz).
const FLY_BOB_FREQUENCY_HZ: f32 = 2.0;

/// Sprite sheet frame width in pixels.
const FRAME_WIDTH_PX: f32 = 8.0;
/// Sprite sheet total width in pixels (2 frames).
const SHEET_WIDTH_PX: f32 = 16.0;
/// Sprite sheet height in pixels.
const SHEET_HEIGHT_PX: f32 = 8.0;
/// Number of animation frames per creature sprite sheet.
const FRAME_COUNT: usize = 2;

// ---------------------------------------------------------------------------
// Creature definitions
// ---------------------------------------------------------------------------

struct CreatureDef {
    path: &'static str,
    movement: MovementType,
    speed: f32,
}

const CITY_CREATURES: &[CreatureDef] = &[
    CreatureDef {
        path: "sprites/creatures/city/mouse.webp",
        movement: MovementType::Ground,
        speed: WANDER_SPEED_PX,
    },
    CreatureDef {
        path: "sprites/creatures/city/pigeon.webp",
        movement: MovementType::Flying,
        speed: WANDER_SPEED_PX,
    },
    CreatureDef {
        path: "sprites/creatures/city/stray_cat.webp",
        movement: MovementType::Ground,
        speed: WANDER_SPEED_PX,
    },
];

const GREENWOOD_CREATURES: &[CreatureDef] = &[
    CreatureDef {
        path: "sprites/creatures/greenwood/butterfly.webp",
        movement: MovementType::Flying,
        speed: WANDER_SPEED_PX,
    },
    CreatureDef {
        path: "sprites/creatures/greenwood/frog.webp",
        movement: MovementType::Ground,
        speed: WANDER_SPEED_PX,
    },
    CreatureDef {
        path: "sprites/creatures/greenwood/rabbit.webp",
        movement: MovementType::Ground,
        speed: WANDER_SPEED_PX,
    },
    CreatureDef {
        path: "sprites/creatures/greenwood/songbird.webp",
        movement: MovementType::Flying,
        speed: WANDER_SPEED_PX,
    },
];

const DARKWOOD_CREATURES: &[CreatureDef] = &[
    CreatureDef {
        path: "sprites/creatures/darkwood/cockroach.webp",
        movement: MovementType::Ground,
        speed: WANDER_SPEED_PX,
    },
    CreatureDef {
        path: "sprites/creatures/darkwood/crow.webp",
        movement: MovementType::Flying,
        speed: WANDER_SPEED_PX,
    },
    CreatureDef {
        path: "sprites/creatures/darkwood/bat.webp",
        movement: MovementType::Flying,
        speed: WANDER_SPEED_PX,
    },
    CreatureDef {
        path: "sprites/creatures/darkwood/spider.webp",
        movement: MovementType::Ground,
        speed: WANDER_SPEED_PX,
    },
];

// ---------------------------------------------------------------------------
// Spawning
// ---------------------------------------------------------------------------

/// Spawn 4-8 creatures for a single area.
pub fn spawn_area_creatures(
    commands: &mut Commands,
    asset_server: &AssetServer,
    atlas_layouts: &mut Assets<TextureAtlasLayout>,
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

    let creature_seed = area_seed.wrapping_add(CREATURE_SEED_SALT);

    // Collect candidate grass tiles (or dirt for city).
    let mut candidates: Vec<(u32, u32)> = Vec::new();
    for x in EDGE_INSET..(MAP_WIDTH - EDGE_INSET) {
        for y in EDGE_INSET..(MAP_HEIGHT - EDGE_INSET) {
            let xu = u32::from(x);
            let yu = u32::from(y);
            let terrain = area.terrain_at(xu, yu);
            if terrain == Some(Terrain::Grass) || terrain == Some(Terrain::Dirt) {
                candidates.push((xu, yu));
            }
        }
    }

    if candidates.is_empty() {
        return;
    }

    // Deterministically shuffle candidates.
    let len = candidates.len();
    let mut rng = u64::from(creature_seed);
    for i in (1..len).rev() {
        rng = lcg(rng);
        #[allow(clippy::as_conversions)]
        let j = (rng % u64::try_from(i + 1).expect("i+1 fits u64")) as usize;
        candidates.swap(i, j);
    }

    // Pick creature count (4-8) deterministically.
    rng = lcg(rng);
    #[allow(clippy::as_conversions)]
    let range = (MAX_CREATURES_PER_AREA - MIN_CREATURES_PER_AREA + 1) as u64;
    #[allow(clippy::as_conversions)]
    let count = MIN_CREATURES_PER_AREA + (rng % range) as usize;
    let count = count.min(candidates.len());

    // Build shared texture atlas layout for 2-frame 8x8 sprite sheets.
    let layout = TextureAtlasLayout::from_grid(
        UVec2::new(
            u32::try_from(FRAME_WIDTH_PX as u16).expect("frame width fits u32"),
            u32::try_from(SHEET_HEIGHT_PX as u16).expect("frame height fits u32"),
        ),
        FRAME_COUNT,
        1,
        None,
        None,
    );
    let layout_handle = atlas_layouts.add(layout);

    for (i, &(xu, yu)) in candidates.iter().take(count).enumerate() {
        let effective_alignment =
            blending::blended_alignment(area.alignment, xu, yu, area_pos, world);
        let biome = Biome::from_alignment(effective_alignment);
        let pool = match biome {
            Biome::City => CITY_CREATURES,
            Biome::Greenwood => GREENWOOD_CREATURES,
            Biome::Darkwood => DARKWOOD_CREATURES,
        };

        let variant = tile_hash(
            xu,
            yu,
            creature_seed.wrapping_add(u32::try_from(i).expect("i fits u32")),
        ) % pool.len();
        let def = &pool[variant];

        let world_x = base_offset_x
            + f32::from(u16::try_from(xu).expect("xu fits u16")) * tile_px
            + tile_px / 2.0;
        let world_y = base_offset_y
            + f32::from(u16::try_from(yu).expect("yu fits u16")) * tile_px
            + tile_px / 2.0;

        let z = Layer::World.z_f32() - world_y * Y_SORT_SCALE;
        let entity_seed = creature_seed
            .wrapping_add(u32::try_from(i).expect("i fits u32"))
            .wrapping_mul(2_654_435_761);

        commands.spawn((
            Creature,
            CreatureAi::new(def.speed, def.movement, entity_seed),
            Sprite {
                image: asset_server.load(def.path),
                texture_atlas: Some(TextureAtlas {
                    layout: layout_handle.clone(),
                    index: 0,
                }),
                ..default()
            },
            Transform::from_xyz(world_x, world_y, z),
        ));
    }
}

// ---------------------------------------------------------------------------
// AI Systems
// ---------------------------------------------------------------------------

/// Transition creature AI states based on timers and player proximity.
pub fn creature_state_transitions(
    time: Res<Time>,
    player_q: Query<&Transform, With<Player>>,
    mut creature_q: Query<(&Transform, &mut CreatureAi), (With<Creature>, Without<Player>)>,
) {
    let player_pos = player_q
        .iter()
        .next()
        .map(|t| t.translation.truncate());

    for (tf, mut ai) in &mut creature_q {
        let creature_pos = tf.translation.truncate();

        // Check flee trigger from any state.
        if let Some(pp) = player_pos {
            let dist = creature_pos.distance(pp);
            if dist < FLEE_TRIGGER_DISTANCE_PX
                && !matches!(ai.state, CreatureState::Flee)
            {
                ai.state = CreatureState::Flee;
                ai.timer = Timer::from_seconds(0.0, TimerMode::Once);
                continue;
            }
            // Check flee exit.
            if matches!(ai.state, CreatureState::Flee) && dist > FLEE_STOP_DISTANCE_PX
            {
                let seed = ai.next_seed();
                let (state, timer) = CreatureState::new_idle(seed);
                ai.state = state;
                ai.timer = timer;
                continue;
            }
        }

        // Timer-based transitions for Idle and Wander.
        ai.timer.tick(time.delta());
        if ai.timer.finished() {
            let seed = ai.next_seed();
            match &ai.state {
                CreatureState::Idle => {
                    let (state, timer) = CreatureState::new_wander(seed);
                    ai.state = state;
                    ai.timer = timer;
                }
                CreatureState::Wander(_) => {
                    let (state, timer) = CreatureState::new_idle(seed);
                    ai.state = state;
                    ai.timer = timer;
                }
                CreatureState::Flee => {
                    // Flee has no timer-based exit -- handled by distance above.
                }
            }
        }
    }
}

/// Apply movement velocity to creatures based on their AI state.
pub fn creature_movement(
    time: Res<Time>,
    player_q: Query<&Transform, With<Player>>,
    mut creature_q: Query<
        (&mut Transform, &CreatureAi),
        (With<Creature>, Without<Player>),
    >,
) {
    let dt = time.delta_secs();
    let player_pos = player_q
        .iter()
        .next()
        .map(|t| t.translation.truncate());

    for (mut tf, ai) in &mut creature_q {
        let velocity = match &ai.state {
            CreatureState::Idle => Vec2::ZERO,
            CreatureState::Wander(dir) => *dir * ai.speed,
            CreatureState::Flee => {
                if let Some(pp) = player_pos {
                    let away = (tf.translation.truncate() - pp).normalize_or_zero();
                    away * FLEE_SPEED_PX
                } else {
                    Vec2::ZERO
                }
            }
        };

        tf.translation.x += velocity.x * dt;
        tf.translation.y += velocity.y * dt;

        // Update z for y-sort.
        tf.translation.z = Layer::World.z_f32() - tf.translation.y * Y_SORT_SCALE;
    }
}

/// Flip ground creature sprites horizontally based on movement direction,
/// and apply vertical bobbing for flying creatures.
pub fn creature_animation(
    time: Res<Time>,
    mut query: Query<(&CreatureAi, &mut Sprite, &mut Transform), With<Creature>>,
) {
    let elapsed = time.elapsed_secs();

    for (ai, mut sprite, mut tf) in &mut query {
        // Set animation frame: 0 for idle, 1 for moving.
        let moving = !matches!(ai.state, CreatureState::Idle);
        if let Some(atlas) = &mut sprite.texture_atlas {
            atlas.index = if moving { 1 } else { 0 };
        }

        match ai.movement {
            MovementType::Ground => {
                // Flip sprite to face movement direction.
                let x_vel = match &ai.state {
                    CreatureState::Wander(dir) => dir.x,
                    CreatureState::Flee => {
                        // Use current transform direction hint -- approximate.
                        0.0
                    }
                    CreatureState::Idle => 0.0,
                };
                if x_vel < 0.0 {
                    sprite.flip_x = true;
                } else if x_vel > 0.0 {
                    sprite.flip_x = false;
                }
            }
            MovementType::Flying => {
                // Apply vertical bobbing.
                if moving {
                    let bob = (elapsed * FLY_BOB_FREQUENCY_HZ).sin()
                        * FLY_BOB_AMPLITUDE_PX;
                    // Store bob in a small y offset. Since we update y every frame
                    // from movement, we apply bob additively via transform hack:
                    // We just offset the visual position slightly. Because z is
                    // recalculated each frame in creature_movement, this is safe.
                    tf.translation.y += bob * time.delta_secs();
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Despawn
// ---------------------------------------------------------------------------

/// Despawn all creatures on game exit.
pub fn despawn_creatures(mut commands: Commands, query: Query<Entity, With<Creature>>) {
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
git add level/src/creatures.rs
git commit -m "Implement creature spawning, AI state machine, and animation"
```

---

## Task 3: Wire Creatures Into Level Systems

**Files:**
- Modify: `level/src/lib.rs`
- Modify: `level/src/spawning.rs`
- Modify: `level/src/plugin.rs`

- [ ] **Step 1: Add creatures module to level/src/lib.rs**

Add in alphabetical order (after `blending`):

```rust
pub mod creatures;
```

- [ ] **Step 2: Call spawn_area_creatures in spawning.rs**

In `level/src/spawning.rs`, add the creatures import at the top:

```rust
use crate::creatures;
```

Then in the `ensure_area_spawned` function, add the creatures spawn call after the grass spawn:

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
    creatures::spawn_area_creatures(commands, asset_server, atlas_layouts, area, area_pos, world);
    spawned.0.insert(area_pos);
}
```

- [ ] **Step 3: Register creature systems in level/src/plugin.rs**

Add `use crate::creatures;` to the imports.

Add creature systems to the `Update` system set (inside `run_if(in_state(GameState::Playing))`):

```rust
creatures::creature_state_transitions,
creatures::creature_movement,
creatures::creature_animation,
```

Add `creatures::despawn_creatures` to the `OnExit(GameState::Playing)` system set (inside `run_if(should_despawn_world)`):

```rust
creatures::despawn_creatures,
```

The full plugin.rs after this change (assumes weather and grass are already wired from prior plans):

```rust
use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::TilemapPlugin;
use models::alignment::PlayerAlignment;
use models::game_states::{should_despawn_world, GameState};
use models::weather::WeatherState;
use models::wind::WindStrength;

use crate::bark_bubbles;
use crate::creatures;
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
                    creatures::creature_state_transitions,
                    creatures::creature_movement,
                    creatures::creature_animation,
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
                    creatures::despawn_creatures,
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
git commit -m "Wire creatures into area spawning and level plugin"
```

---

## Task 4: Generate Creature Sprite Sheets via PixelLab

**Files:**
- Create: 11 sprite sheets in `assets/sprites/creatures/`

All sprites use the `create_map_object` PixelLab tool. Each is a 16x8 sheet with two 8x8 frames side by side (idle left, move right). Style suffix for all: "Warm earthy palette with hue-shifted shadows toward cool purple and highlights toward warm gold. Moderate saturation, clean readable forms, storybook fantasy RPG style. 16-bit pixel art aesthetic."

- [ ] **Step 1: Generate city/mouse.webp**

Use `mcp__pixellab__create_map_object` with:
- `description`: "A tiny gray mouse sprite sheet, two frames side by side: left frame idle sitting, right frame running with legs extended. Top-down view, small and cute. Warm earthy palette with hue-shifted shadows toward cool purple and highlights toward warm gold. Moderate saturation, clean readable forms, storybook fantasy RPG style. 16-bit pixel art aesthetic."
- `view`: "low top-down"
- `outline`: "single color outline"
- `shading`: "basic shading"
- `detail`: "medium detail"
- `width`: 16
- `height`: 8
- `padding`: 4

Save to `assets/sprites/creatures/city/mouse.webp`.

- [ ] **Step 2: Generate city/pigeon.webp**

Same params but:
- `description`: "A tiny gray-blue pigeon sprite sheet, two frames side by side: left frame idle standing, right frame wings spread flying. Top-down view, small bird. Warm earthy palette with hue-shifted shadows toward cool purple and highlights toward warm gold. Moderate saturation, clean readable forms, storybook fantasy RPG style. 16-bit pixel art aesthetic."

Save to `assets/sprites/creatures/city/pigeon.webp`.

- [ ] **Step 3: Generate city/stray_cat.webp**

Same params but:
- `description`: "A tiny orange tabby stray cat sprite sheet, two frames side by side: left frame idle sitting with tail curled, right frame walking with legs extended. Top-down view. Warm earthy palette with hue-shifted shadows toward cool purple and highlights toward warm gold. Moderate saturation, clean readable forms, storybook fantasy RPG style. 16-bit pixel art aesthetic."

Save to `assets/sprites/creatures/city/stray_cat.webp`.

- [ ] **Step 4: Generate greenwood/butterfly.webp**

Same params but:
- `description`: "A tiny colorful butterfly sprite sheet, two frames side by side: left frame wings closed, right frame wings open. Top-down view, bright blue-yellow wings. Warm earthy palette with hue-shifted shadows toward cool purple and highlights toward warm gold. Moderate saturation, clean readable forms, storybook fantasy RPG style. 16-bit pixel art aesthetic."

Save to `assets/sprites/creatures/greenwood/butterfly.webp`.

- [ ] **Step 5: Generate greenwood/frog.webp**

Same params but:
- `description`: "A tiny green frog sprite sheet, two frames side by side: left frame idle sitting, right frame mid-hop with legs extended. Top-down view. Warm earthy palette with hue-shifted shadows toward cool purple and highlights toward warm gold. Moderate saturation, clean readable forms, storybook fantasy RPG style. 16-bit pixel art aesthetic."

Save to `assets/sprites/creatures/greenwood/frog.webp`.

- [ ] **Step 6: Generate greenwood/rabbit.webp**

Same params but:
- `description`: "A tiny brown rabbit sprite sheet, two frames side by side: left frame idle sitting with ears up, right frame hopping with legs extended. Top-down view. Warm earthy palette with hue-shifted shadows toward cool purple and highlights toward warm gold. Moderate saturation, clean readable forms, storybook fantasy RPG style. 16-bit pixel art aesthetic."

Save to `assets/sprites/creatures/greenwood/rabbit.webp`.

- [ ] **Step 7: Generate greenwood/songbird.webp**

Same params but:
- `description`: "A tiny yellow-red songbird sprite sheet, two frames side by side: left frame perched idle, right frame wings spread flying. Top-down view. Warm earthy palette with hue-shifted shadows toward cool purple and highlights toward warm gold. Moderate saturation, clean readable forms, storybook fantasy RPG style. 16-bit pixel art aesthetic."

Save to `assets/sprites/creatures/greenwood/songbird.webp`.

- [ ] **Step 8: Generate darkwood/cockroach.webp**

Same params but:
- `description`: "A tiny dark brown cockroach sprite sheet, two frames side by side: left frame idle with antennae forward, right frame scurrying with legs extended. Top-down view. Warm earthy palette with hue-shifted shadows toward cool purple and highlights toward warm gold. Moderate saturation, clean readable forms, storybook fantasy RPG style. 16-bit pixel art aesthetic."

Save to `assets/sprites/creatures/darkwood/cockroach.webp`.

- [ ] **Step 9: Generate darkwood/crow.webp**

Same params but:
- `description`: "A tiny black crow sprite sheet, two frames side by side: left frame idle standing, right frame wings spread flying. Top-down view, dark ominous bird. Warm earthy palette with hue-shifted shadows toward cool purple and highlights toward warm gold. Moderate saturation, clean readable forms, storybook fantasy RPG style. 16-bit pixel art aesthetic."

Save to `assets/sprites/creatures/darkwood/crow.webp`.

- [ ] **Step 10: Generate darkwood/bat.webp**

Same params but:
- `description`: "A tiny dark purple-brown bat sprite sheet, two frames side by side: left frame wings folded, right frame wings spread flying. Top-down view. Warm earthy palette with hue-shifted shadows toward cool purple and highlights toward warm gold. Moderate saturation, clean readable forms, storybook fantasy RPG style. 16-bit pixel art aesthetic."

Save to `assets/sprites/creatures/darkwood/bat.webp`.

- [ ] **Step 11: Generate darkwood/spider.webp**

Same params but:
- `description`: "A tiny black spider sprite sheet, two frames side by side: left frame idle with legs tucked, right frame crawling with legs extended. Top-down view, eight visible legs. Warm earthy palette with hue-shifted shadows toward cool purple and highlights toward warm gold. Moderate saturation, clean readable forms, storybook fantasy RPG style. 16-bit pixel art aesthetic."

Save to `assets/sprites/creatures/darkwood/spider.webp`.

- [ ] **Step 12: Commit**

```bash
git add assets/sprites/creatures/
git commit -m "Add creature sprite sheets for all three biomes (11 sheets)"
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
