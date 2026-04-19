# Phase 3: Polish (Weather + Shadows + Dither) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add atmospheric weather particles (firefly, dust mote, fog patch), procedural drop shadows under all entities, and shader-level Bayer dither in the biome atmosphere shader.

**Architecture:** Extend existing crates -- `models/` (data), `level/` (spawning + helpers), `player/` (one shadow call site). All three particle variants use procedural `Mesh2d(Circle::new(1.0))` + `MeshMaterial2d<ColorMaterial>` (no new sprite assets). Drop shadows share one mesh + one material handle for batched rendering. Bayer dither folded into the existing `BiomeAtmosphere` fragment shader (no new pipeline pass).

**Tech Stack:** Bevy 0.18.1 (`Mesh2d`, `MeshMaterial2d`, `ColorMaterial`), Rust nightly, WASM. New module: `models/src/shadow.rs`, `level/src/shadows.rs`. Modifies: `models::weather`, `models::palette`, `level::weather`, `level::scenery`, `level::npcs`, `level::grass`, `level::creatures`, `level::galen`, `level::plugin`, `player::spawning`, `assets/shaders/biome_atmosphere.wgsl`.

**Asset decision (locked at plan time):** All three new particle variants render via `Mesh2d(Circle)` + `ColorMaterial` -- no new WebP files needed. Firefly emissive color >1.0 still triggers HDR bloom. Fog uses a low-alpha color on a stretched ellipse.

---

## File Structure

**Create:**
- `models/src/shadow.rs` -- per-asset shadow geometry (Vec2 + f32 constants).
- `level/src/shadows.rs` -- `DropShadowAssets` resource, `init_shadow_assets` startup system, `spawn_drop_shadow` helper.

**Modify:**
- `models/src/lib.rs` -- declare `pub mod shadow;`.
- `models/src/weather.rs` -- extend `ParticleVariant` enum.
- `models/src/palette.rs` -- add 4 new color constants.
- `level/src/lib.rs` -- declare `pub mod shadows;`.
- `level/src/weather.rs` -- 3 predicates, 3 spawning systems, 1 firefly pulse animation system. Helpers for procedural particle spawn (Mesh2d-based).
- `level/src/plugin.rs` -- register `init_shadow_assets` (Startup) and the new particle systems.
- `level/src/scenery.rs` -- call `spawn_drop_shadow` for trees.
- `level/src/npcs.rs` -- call `spawn_drop_shadow` for NPCs.
- `level/src/grass.rs` -- call `spawn_drop_shadow` for grass.
- `level/src/creatures.rs` -- call `spawn_drop_shadow` for creatures.
- `level/src/galen.rs` -- call `spawn_drop_shadow` for Galen.
- `player/src/spawning.rs` -- call `spawn_drop_shadow` for player.
- `player/Cargo.toml` -- add `level` dep if not present (verify at task time).
- `assets/shaders/biome_atmosphere.wgsl` -- add Bayer 4x4 dither.

---

## Task 1: Pure data scaffolding (palette + ParticleVariant + shadow geometry)

Goal: add all data constants and enum variants up front so behavior tasks can reference them without ordering churn. No behavior change yet.

**Files:**
- Modify: `models/src/palette.rs`
- Modify: `models/src/weather.rs`
- Create: `models/src/shadow.rs`
- Modify: `models/src/lib.rs`

- [ ] **Step 1: Add palette constants**

Edit `/home/ddudson/repos/evergreen/models/src/palette.rs`. Append at the bottom:

```rust
// Weather particle colors.
pub const FIREFLY: Color = Color::srgb(2.0, 3.0, 0.5);
pub const DUST_MOTE: Color = Color::srgb(0.9, 0.85, 0.75);
pub const FOG: Color = Color::srgba(0.6, 0.65, 0.7, 0.35);

// Drop shadow color.
pub const DROP_SHADOW: Color = Color::srgba(0.0, 0.0, 0.0, 0.4);
```

- [ ] **Step 2: Extend ParticleVariant**

Edit `/home/ddudson/repos/evergreen/models/src/weather.rs`. Find the `pub enum ParticleVariant { ... }` block and add three variants:

```rust
pub enum ParticleVariant {
    GreenLeaf,
    BrownLeaf,
    PaperScrap,
    Raindrop,
    Splash,
    Firefly,
    DustMote,
    FogPatch,
}
```

- [ ] **Step 3: Create models/src/shadow.rs**

Create `/home/ddudson/repos/evergreen/models/src/shadow.rs`:

```rust
//! Per-asset drop-shadow geometry. Used by `level::shadows::spawn_drop_shadow`.

use bevy::math::Vec2;

pub const PLAYER_SHADOW_HALF_PX: Vec2 = Vec2::new(8.0, 3.0);
pub const PLAYER_SHADOW_OFFSET_Y_PX: f32 = -10.0;

pub const NPC_SHADOW_HALF_PX: Vec2 = Vec2::new(7.0, 2.5);
pub const NPC_SHADOW_OFFSET_Y_PX: f32 = -10.0;

pub const TREE_SHADOW_HALF_PX: Vec2 = Vec2::new(18.0, 5.0);
/// Tree sprite is anchored at BOTTOM_CENTER, so the entity Transform sits
/// at the trunk base -- shadow rests at the same y.
pub const TREE_SHADOW_OFFSET_Y_PX: f32 = 0.0;

pub const GRASS_SHADOW_HALF_PX: Vec2 = Vec2::new(4.0, 1.5);
pub const GRASS_SHADOW_OFFSET_Y_PX: f32 = -2.0;

pub const CREATURE_SHADOW_HALF_PX: Vec2 = Vec2::new(3.0, 1.0);
pub const CREATURE_SHADOW_OFFSET_Y_PX: f32 = -3.0;

pub const GALEN_SHADOW_HALF_PX: Vec2 = Vec2::new(7.0, 2.5);
pub const GALEN_SHADOW_OFFSET_Y_PX: f32 = -10.0;
```

- [ ] **Step 4: Declare module in models/src/lib.rs**

Edit `/home/ddudson/repos/evergreen/models/src/lib.rs`. Add `pub mod shadow;` in alphabetical order alongside existing module declarations.

- [ ] **Step 5: Build + clippy + tests**

Run from `/home/ddudson/repos/evergreen`:
- `cargo build`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace --target x86_64-unknown-linux-gnu --lib`

All must pass. Pre-existing keybinds doctest failure OUT OF SCOPE.

If clippy warns "unused variant" on the new `Firefly`/`DustMote`/`FogPatch` enum members, that's expected (no consumer yet) and should NOT fire because Rust's `dead_code` lint allows unused enum variants by default. If it does fire, a `#[allow(dead_code)]` on the enum is acceptable -- they get used in T2-T5.

- [ ] **Step 6: Commit**

```bash
git add models/
git commit -m "feat(models): polish data scaffolding (particle variants + shadow geometry + palette)"
```

---

## Task 2: Firefly predicate + spawning + 4 unit tests (TDD)

Goal: pure predicate `firefly_active(hour, alignment)` tested in isolation; spawn helper that creates Mesh2d firefly entities; spawning system that runs every frame in `Playing` and emits particles when the predicate is true.

**Files:**
- Modify: `level/src/weather.rs` (predicate + spawn helper + spawning system + 4 tests)
- Modify: `level/src/plugin.rs` (register new system)

- [ ] **Step 1: Write the failing tests (and predicate definition)**

Edit `/home/ddudson/repos/evergreen/level/src/weather.rs`. Append the predicate and tests at the end of the file:

```rust
// ---------------------------------------------------------------------------
// Firefly
// ---------------------------------------------------------------------------

use models::area::AreaAlignment;

/// Firefly threshold: alignment > this AND hour outside daylight enables fireflies.
const FIREFLY_ALIGNMENT_THRESHOLD: AreaAlignment = 60;
/// Hour-of-day before which fireflies are active (early morning).
const FIREFLY_HOUR_START: f32 = 5.0;
/// Hour-of-day after which fireflies are active (post-dusk).
const FIREFLY_HOUR_END: f32 = 19.0;

/// Pure predicate: should fireflies spawn for this hour + alignment?
pub fn firefly_active(hour: f32, alignment: AreaAlignment) -> bool {
    alignment > FIREFLY_ALIGNMENT_THRESHOLD
        && !(FIREFLY_HOUR_START..=FIREFLY_HOUR_END).contains(&hour)
}

#[cfg(test)]
mod firefly_tests {
    use super::*;

    #[test]
    fn firefly_active_at_night_in_darkwood() {
        assert!(firefly_active(22.0, 80));
    }

    #[test]
    fn firefly_inactive_at_midday() {
        assert!(!firefly_active(12.0, 80));
    }

    #[test]
    fn firefly_inactive_in_city_at_night() {
        assert!(!firefly_active(22.0, 10));
    }

    #[test]
    fn firefly_threshold_boundary_strict() {
        // alignment == threshold => false (strict >)
        assert!(!firefly_active(22.0, 60));
    }
}
```

NOTE: `AreaAlignment` lives at `level::area::AreaAlignment` (a `pub type AreaAlignment = u8;`). The import is from the same crate, so use `use crate::area::AreaAlignment;` instead of `use models::area::...`. Verify before saving.

- [ ] **Step 2: Run the tests**

```bash
cargo test -p level --lib --target x86_64-unknown-linux-gnu firefly
```

Expected: 4 tests pass.

- [ ] **Step 3: Add the spawning system + Mesh2d helper**

Append to `/home/ddudson/repos/evergreen/level/src/weather.rs` (above the `firefly_tests` module):

```rust
use bevy::sprite::{ColorMaterial, MeshMaterial2d};
use models::palette::FIREFLY;

/// Fireflies per second when active.
const FIREFLIES_PER_SEC: f32 = 0.8;
/// Firefly lifetime.
const FIREFLY_LIFETIME_SECS: f32 = 6.0;
/// Firefly horizontal drift speed (pixels/sec, randomized in ±this range).
const FIREFLY_DRIFT_PX: f32 = 20.0;
/// Firefly visual size in world pixels (one side).
const FIREFLY_SIZE_PX: f32 = 2.0;

/// Per-frame system: spawn fireflies at night in darkwood-leaning biomes.
pub fn spawn_fireflies(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    clock: Res<GameClock>,
    camera_q: Query<&Transform, With<Camera2d>>,
    time: Res<Time>,
    world: Res<WorldMap>,
) {
    let alignment = world.get_area(world.current).map_or(50, |a| a.alignment);
    if !firefly_active(clock.hour, alignment) {
        return;
    }
    let Ok(cam_tf) = camera_q.single() else {
        return;
    };
    let cam_pos = cam_tf.translation.truncate();

    let dt = time.delta_secs();
    let frame_seed = f32_to_seed(time.elapsed_secs()).wrapping_add(31415);
    let count = fractional_to_count(FIREFLIES_PER_SEC * dt, frame_seed);

    for i in 0..count {
        let s = frame_seed.wrapping_add(i).wrapping_add(2718);
        spawn_firefly(&mut commands, &mut meshes, &mut materials, cam_pos, s);
    }
}

fn spawn_firefly(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
    cam_pos: Vec2,
    seed: u32,
) {
    let x_offset = hash_f32(seed, VIEWPORT_HALF_W_PX);
    let y_offset = hash_f32(seed.wrapping_add(1), VIEWPORT_HALF_H_PX);
    let drift = (hash_frac(seed.wrapping_add(2)) - 0.5) * 2.0 * FIREFLY_DRIFT_PX;

    let mesh = meshes.add(Circle::new(1.0));
    let material = materials.add(ColorMaterial::from(FIREFLY));

    commands.spawn((
        WeatherParticle {
            velocity: Vec2::new(drift, 0.0),
            lifetime: Timer::from_seconds(FIREFLY_LIFETIME_SECS, TimerMode::Once),
            variant: ParticleVariant::Firefly,
        },
        Mesh2d(mesh),
        MeshMaterial2d(material),
        Transform::from_translation(Vec3::new(
            cam_pos.x + x_offset,
            cam_pos.y + y_offset,
            Layer::Weather.z_f32(),
        ))
        .with_scale(Vec3::new(FIREFLY_SIZE_PX / 2.0, FIREFLY_SIZE_PX / 2.0, 1.0)),
    ));
}
```

NOTE: `hash_f32`, `hash_frac`, `f32_to_seed`, `fractional_to_count` are existing private helpers in this file -- reuse them. `WeatherParticle`, `ParticleVariant`, `Layer`, `WorldMap`, `GameClock` are all already imported at the top of the file from earlier code.

NOTE: A fresh mesh + material is allocated PER FIREFLY -- not ideal but matches the existing leaf/raindrop pattern (`asset_server.load(path)` is similarly called per particle, and `Assets` deduplicates handles by content where possible). Optimize later if perf demands. The spec calls for one shared mesh+material across all entities for shadows; for particles we tolerate per-entity allocation since the rate is low (<1/sec).

- [ ] **Step 4: Register `spawn_fireflies` in `LevelPlugin`**

Read `/home/ddudson/repos/evergreen/level/src/plugin.rs` and locate where existing weather systems (`spawn_weather_particles`, `update_weather_particles`) are registered. Add `spawn_fireflies` to the same `Update` system tuple, gated on the same state condition.

If the existing block looks like:

```rust
.add_systems(
    Update,
    (
        weather_state_machine,
        sync_wind_strength,
        spawn_weather_particles,
        update_weather_particles,
    )
        .run_if(in_state(GameState::Playing)),
);
```

Add `spawn_fireflies` to the tuple. Import: `use crate::weather::spawn_fireflies;` (or however weather systems are currently imported).

- [ ] **Step 5: Build + clippy + tests**

```bash
cargo build
cargo clippy --workspace --all-targets -- -D warnings
cargo test -p level --lib --target x86_64-unknown-linux-gnu firefly
```

All must pass. 4 firefly tests + everything else green.

- [ ] **Step 6: Commit**

```bash
git add level/
git commit -m "feat(weather): firefly particles spawn at night in darkwood"
```

---

## Task 3: Firefly pulse-flicker animation

Goal: each firefly's alpha pulses sinusoidally so they twinkle in/out of bloom threshold.

**Files:**
- Modify: `level/src/weather.rs` (animation system)
- Modify: `level/src/plugin.rs` (register system)

- [ ] **Step 1: Add animation system**

Append to `/home/ddudson/repos/evergreen/level/src/weather.rs` (above the test module):

```rust
/// Firefly pulse frequency (Hz).
const FIREFLY_PULSE_FREQ_HZ: f32 = 2.5;
/// Firefly pulse alpha range: [BASE - AMP, BASE + AMP].
const FIREFLY_PULSE_BASE: f32 = 0.6;
const FIREFLY_PULSE_AMP: f32 = 0.4;

/// Per-frame system: pulse each firefly's material alpha.
pub fn animate_fireflies(
    time: Res<Time>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    query: Query<(Entity, &MeshMaterial2d<ColorMaterial>, &WeatherParticle)>,
) {
    let elapsed = time.elapsed_secs();
    for (entity, mat_handle, particle) in &query {
        if particle.variant != ParticleVariant::Firefly {
            continue;
        }
        // Per-entity phase derived from entity bits so each firefly pulses out of sync.
        let phase = entity_phase(entity);
        let pulse = FIREFLY_PULSE_BASE
            + FIREFLY_PULSE_AMP * (elapsed * FIREFLY_PULSE_FREQ_HZ + phase).sin();
        if let Some(mat) = materials.get_mut(&mat_handle.0) {
            mat.color = mat.color.with_alpha(pulse);
        }
    }
}

fn entity_phase(entity: Entity) -> f32 {
    let bits = entity.to_bits();
    #[allow(clippy::as_conversions)]
    let frac = ((bits.wrapping_mul(2_654_435_761) % 10_000) as f32) / 10_000.0;
    frac * std::f32::consts::TAU
}
```

NOTE: `Color::with_alpha` exists on Bevy 0.18 `Color`. If clippy or rustc errors with a different method name (e.g. `set_alpha`), inspect Bevy's `Color` API and adjust.

NOTE: The `#[allow(clippy::as_conversions)]` is acceptable here -- we're converting a u64 → f32 for hash diffusion, which is intrinsically lossy and the `as` is the cleanest expression.

- [ ] **Step 2: Register `animate_fireflies` in LevelPlugin**

Edit `/home/ddudson/repos/evergreen/level/src/plugin.rs`. Add `animate_fireflies` to the same `Update` tuple as `spawn_fireflies`. Import: `use crate::weather::animate_fireflies;`.

- [ ] **Step 3: Build + clippy + tests**

```bash
cargo build
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --target x86_64-unknown-linux-gnu --lib
```

All must pass.

- [ ] **Step 4: Commit**

```bash
git add level/
git commit -m "feat(weather): firefly pulse-flicker animation"
```

---

## Task 4: Dust mote predicate + spawning + 2 unit tests (TDD)

Goal: subtle floating dust during clear weather, lit by ambient.

**Files:**
- Modify: `level/src/weather.rs` (predicate + spawn + 2 tests)
- Modify: `level/src/plugin.rs` (register system)

- [ ] **Step 1: Write the failing tests + predicate**

Append to `/home/ddudson/repos/evergreen/level/src/weather.rs` (before the existing test module):

```rust
// ---------------------------------------------------------------------------
// Dust motes
// ---------------------------------------------------------------------------

/// Dust motes per second during clear weather.
const DUST_MOTES_PER_SEC: f32 = 1.5;
/// Dust mote lifetime.
const DUST_LIFETIME_SECS: f32 = 8.0;
/// Dust mote drift speed (pixels/sec).
const DUST_DRIFT_SPEED_PX: f32 = 10.0;
/// Dust mote visual size (one side).
const DUST_SIZE_PX: f32 = 1.0;

/// Pure predicate: should dust motes spawn for this weather kind?
pub fn dust_mote_active(weather: WeatherKind) -> bool {
    matches!(weather, WeatherKind::Clear)
}

#[cfg(test)]
mod dust_mote_tests {
    use super::*;

    #[test]
    fn dust_mote_active_during_clear() {
        assert!(dust_mote_active(WeatherKind::Clear));
    }

    #[test]
    fn dust_mote_inactive_during_rain() {
        assert!(!dust_mote_active(WeatherKind::Rain));
    }
}
```

- [ ] **Step 2: Run the tests**

```bash
cargo test -p level --lib --target x86_64-unknown-linux-gnu dust_mote
```

Expected: 2 tests pass.

- [ ] **Step 3: Add spawning system**

Append (above the dust_mote_tests):

```rust
use models::palette::DUST_MOTE;

/// Per-frame system: spawn dust motes during clear weather.
pub fn spawn_dust_motes(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    weather: Res<WeatherState>,
    wind_dir: Res<WindDirection>,
    camera_q: Query<&Transform, With<Camera2d>>,
    time: Res<Time>,
) {
    if !dust_mote_active(weather.current) {
        return;
    }
    let Ok(cam_tf) = camera_q.single() else {
        return;
    };
    let cam_pos = cam_tf.translation.truncate();
    let dir = wind_dir.as_vec2();

    let dt = time.delta_secs();
    let frame_seed = f32_to_seed(time.elapsed_secs()).wrapping_add(99991);
    let count = fractional_to_count(DUST_MOTES_PER_SEC * dt, frame_seed);

    for i in 0..count {
        let s = frame_seed.wrapping_add(i).wrapping_add(31415);
        spawn_dust_mote(&mut commands, &mut meshes, &mut materials, cam_pos, s, dir);
    }
}

fn spawn_dust_mote(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
    cam_pos: Vec2,
    seed: u32,
    wind_dir: Vec2,
) {
    let x_offset = hash_f32(seed, VIEWPORT_HALF_W_PX);
    let y_offset = hash_f32(seed.wrapping_add(1), VIEWPORT_HALF_H_PX);
    let velocity = wind_dir * DUST_DRIFT_SPEED_PX;

    let mesh = meshes.add(Circle::new(1.0));
    let material = materials.add(ColorMaterial::from(DUST_MOTE));

    commands.spawn((
        WeatherParticle {
            velocity,
            lifetime: Timer::from_seconds(DUST_LIFETIME_SECS, TimerMode::Once),
            variant: ParticleVariant::DustMote,
        },
        Mesh2d(mesh),
        MeshMaterial2d(material),
        Transform::from_translation(Vec3::new(
            cam_pos.x + x_offset,
            cam_pos.y + y_offset,
            Layer::Weather.z_f32(),
        ))
        .with_scale(Vec3::new(DUST_SIZE_PX / 2.0, DUST_SIZE_PX / 2.0, 1.0)),
    ));
}
```

- [ ] **Step 4: Register `spawn_dust_motes` in LevelPlugin**

Edit `/home/ddudson/repos/evergreen/level/src/plugin.rs`. Add `spawn_dust_motes` to the same `Update` tuple. Import: `use crate::weather::spawn_dust_motes;`.

- [ ] **Step 5: Build + clippy + tests**

```bash
cargo build
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --target x86_64-unknown-linux-gnu --lib
```

All must pass.

- [ ] **Step 6: Commit**

```bash
git add level/
git commit -m "feat(weather): dust mote particles during clear weather"
```

---

## Task 5: Fog patch predicate + spawning + 2 unit tests (TDD)

Goal: low-alpha drifting fog patches in darkwood biomes for atmospheric overlay.

**Files:**
- Modify: `level/src/weather.rs` (predicate + spawn + 2 tests)
- Modify: `level/src/plugin.rs` (register system)

- [ ] **Step 1: Write the failing tests + predicate**

Append to `/home/ddudson/repos/evergreen/level/src/weather.rs` (before the existing test modules):

```rust
// ---------------------------------------------------------------------------
// Fog
// ---------------------------------------------------------------------------

/// Fog patches per second in darkwood.
const FOG_PATCHES_PER_SEC: f32 = 0.3;
/// Fog patch lifetime.
const FOG_LIFETIME_SECS: f32 = 12.0;
/// Fog patch drift speed (pixels/sec).
const FOG_DRIFT_SPEED_PX: f32 = 15.0;
/// Fog patch ellipse half-size (x, y) in world pixels.
const FOG_HALF_PX_X: f32 = 16.0;
const FOG_HALF_PX_Y: f32 = 8.0;
/// Alignment threshold above which fog spawns.
const FOG_ALIGNMENT_THRESHOLD: AreaAlignment = 75;

/// Pure predicate: should fog patches spawn for this alignment?
pub fn fog_active(alignment: AreaAlignment) -> bool {
    alignment > FOG_ALIGNMENT_THRESHOLD
}

#[cfg(test)]
mod fog_tests {
    use super::*;

    #[test]
    fn fog_active_in_darkwood() {
        assert!(fog_active(80));
    }

    #[test]
    fn fog_inactive_in_greenwood() {
        assert!(!fog_active(50));
    }
}
```

- [ ] **Step 2: Run the tests**

```bash
cargo test -p level --lib --target x86_64-unknown-linux-gnu fog
```

Expected: 2 tests pass.

- [ ] **Step 3: Add spawning system**

Append (above the fog_tests):

```rust
use models::palette::FOG;

/// Per-frame system: spawn fog patches in darkwood areas.
pub fn spawn_fog_patches(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    wind_dir: Res<WindDirection>,
    camera_q: Query<&Transform, With<Camera2d>>,
    time: Res<Time>,
    world: Res<WorldMap>,
) {
    let alignment = world.get_area(world.current).map_or(50, |a| a.alignment);
    if !fog_active(alignment) {
        return;
    }
    let Ok(cam_tf) = camera_q.single() else {
        return;
    };
    let cam_pos = cam_tf.translation.truncate();
    let dir = wind_dir.as_vec2();

    let dt = time.delta_secs();
    let frame_seed = f32_to_seed(time.elapsed_secs()).wrapping_add(77777);
    let count = fractional_to_count(FOG_PATCHES_PER_SEC * dt, frame_seed);

    for i in 0..count {
        let s = frame_seed.wrapping_add(i).wrapping_add(11111);
        spawn_fog_patch(&mut commands, &mut meshes, &mut materials, cam_pos, s, dir);
    }
}

fn spawn_fog_patch(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
    cam_pos: Vec2,
    seed: u32,
    wind_dir: Vec2,
) {
    let x_offset = hash_f32(seed, VIEWPORT_HALF_W_PX);
    let y_offset = hash_f32(seed.wrapping_add(1), VIEWPORT_HALF_H_PX);
    let velocity = wind_dir * FOG_DRIFT_SPEED_PX;

    let mesh = meshes.add(Circle::new(1.0));
    let material = materials.add(ColorMaterial::from(FOG));

    commands.spawn((
        WeatherParticle {
            velocity,
            lifetime: Timer::from_seconds(FOG_LIFETIME_SECS, TimerMode::Once),
            variant: ParticleVariant::FogPatch,
        },
        Mesh2d(mesh),
        MeshMaterial2d(material),
        Transform::from_translation(Vec3::new(
            cam_pos.x + x_offset,
            cam_pos.y + y_offset,
            Layer::Weather.z_f32(),
        ))
        .with_scale(Vec3::new(FOG_HALF_PX_X, FOG_HALF_PX_Y, 1.0)),
    ));
}
```

- [ ] **Step 4: Register `spawn_fog_patches`**

Edit `/home/ddudson/repos/evergreen/level/src/plugin.rs`. Add `spawn_fog_patches` to the same `Update` tuple. Import: `use crate::weather::spawn_fog_patches;`.

- [ ] **Step 5: Build + clippy + tests**

```bash
cargo build
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --target x86_64-unknown-linux-gnu --lib
```

All must pass. Total weather tests: 4 firefly + 2 dust + 2 fog = 8.

- [ ] **Step 6: Commit**

```bash
git add level/
git commit -m "feat(weather): fog patches drift through darkwood"
```

---

## Task 6: DropShadowAssets resource + helper + startup wire

Goal: shared `Mesh2d` ellipse + `ColorMaterial` for all drop shadows. One handle pair, batched render.

**Files:**
- Create: `level/src/shadows.rs`
- Modify: `level/src/lib.rs` (declare module)
- Modify: `level/src/plugin.rs` (register `init_shadow_assets` on Startup)

- [ ] **Step 1: Create `level/src/shadows.rs`**

Create `/home/ddudson/repos/evergreen/level/src/shadows.rs`:

```rust
//! Shared `Mesh2d` ellipse + `ColorMaterial` handle resource for drop shadows,
//! plus a single `spawn_drop_shadow` helper used by all asset spawn sites.

use bevy::math::Vec2;
use bevy::prelude::*;
use bevy::sprite::{ColorMaterial, MeshMaterial2d};
use models::palette::DROP_SHADOW;

/// Z-offset placing shadow just under its parent sprite (same layer).
const SHADOW_Z_OFFSET: f32 = -0.1;

/// Shared shadow assets. Spawned at `Startup` once, reused by every shadow.
#[derive(Resource)]
pub struct DropShadowAssets {
    pub mesh: Handle<Mesh>,
    pub material: Handle<ColorMaterial>,
}

/// Startup system: build the shared ellipse mesh + dark material.
pub fn init_shadow_assets(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let mesh = meshes.add(Circle::new(1.0));
    let material = materials.add(ColorMaterial::from(DROP_SHADOW));
    commands.insert_resource(DropShadowAssets { mesh, material });
}

/// Spawn one drop shadow as a child of `parent`. The shared circle mesh is
/// scaled by `half_size` to form an ellipse, offset down by `ground_offset_y`.
pub fn spawn_drop_shadow(
    commands: &mut Commands,
    assets: &DropShadowAssets,
    parent: Entity,
    half_size: Vec2,
    ground_offset_y: f32,
) {
    commands.spawn((
        Mesh2d(assets.mesh.clone()),
        MeshMaterial2d(assets.material.clone()),
        Transform::from_translation(Vec3::new(0.0, ground_offset_y, SHADOW_Z_OFFSET))
            .with_scale(half_size.extend(1.0)),
        ChildOf(parent),
    ));
}
```

- [ ] **Step 2: Declare module in `level/src/lib.rs`**

Edit `/home/ddudson/repos/evergreen/level/src/lib.rs`. Add `pub mod shadows;` in alphabetical order.

- [ ] **Step 3: Register `init_shadow_assets` on Startup in LevelPlugin**

Edit `/home/ddudson/repos/evergreen/level/src/plugin.rs`. Find existing `Startup` system registration (e.g. world generation). Add `init_shadow_assets` to the Startup chain:

```rust
.add_systems(Startup, (existing_startup_systems, shadows::init_shadow_assets))
```

If LevelPlugin has no Startup chain, add one. Import: `use crate::shadows;` (or direct path).

- [ ] **Step 4: Build + clippy + tests**

```bash
cargo build
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --target x86_64-unknown-linux-gnu --lib
```

All must pass. No new tests.

- [ ] **Step 5: Commit**

```bash
git add level/
git commit -m "feat(shadows): shared Mesh2d ellipse resource + spawn helper"
```

---

## Task 7: Wire shadow into player spawn

Goal: visible shadow under the player. First spawn-site wiring -- single entity, easy verification.

**Files:**
- Modify: `player/Cargo.toml` (verify `level` dep)
- Modify: `player/src/spawning.rs`

- [ ] **Step 1: Verify `level` dep in player/Cargo.toml**

Read `/home/ddudson/repos/evergreen/player/Cargo.toml`. If it does NOT already list `level = { path = "../level" }`, add it under `[dependencies]`.

- [ ] **Step 2: Wire shadow into spawn**

Edit `/home/ddudson/repos/evergreen/player/src/spawning.rs`. Modify `spawn` function to capture `.id()` and call `spawn_drop_shadow`:

Current spawn:
```rust
commands.spawn((
    Player,
    PLAYER_SPEED,
    PLAYER_MAX_HEALTH,
    FacingDirection::default(),
    AnimationKind::default(),
    AnimationFrame::default(),
    AnimationTimer::default(),
    Sprite { ... },
    Transform::from_xyz(...),
));
```

Modify to:
```rust
let parent = commands
    .spawn((
        Player,
        PLAYER_SPEED,
        PLAYER_MAX_HEALTH,
        FacingDirection::default(),
        AnimationKind::default(),
        AnimationFrame::default(),
        AnimationTimer::default(),
        Sprite {
            image: asset_server.load("sprites/player/briar_sheet.webp"),
            texture_atlas: Some(TextureAtlas {
                layout: layout_handle,
                index: 0,
            }),
            custom_size: Some(tile_size(PLAYER_WIDTH, PLAYER_HEIGHT)),
            ..default()
        },
        Transform::from_xyz(
            area_world_offset(world.current).x,
            area_world_offset(world.current).y,
            Layer::World.z_f32(),
        ),
    ))
    .id();

spawn_drop_shadow(
    &mut commands,
    &shadow_assets,
    parent,
    PLAYER_SHADOW_HALF_PX,
    PLAYER_SHADOW_OFFSET_Y_PX,
);
```

Add to the `spawn` function signature:
```rust
shadow_assets: Res<DropShadowAssets>,
```

Add imports at top of file:
```rust
use level::shadows::{spawn_drop_shadow, DropShadowAssets};
use models::shadow::{PLAYER_SHADOW_HALF_PX, PLAYER_SHADOW_OFFSET_Y_PX};
```

- [ ] **Step 3: Build + clippy + tests**

```bash
cargo build
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --target x86_64-unknown-linux-gnu --lib
```

All must pass. If `Res<DropShadowAssets>` complains about ordering -- the resource must be initialized before the player spawn system runs. Player spawns on `OnEnter(GameState::Playing)` which runs after `Startup`. Resource is inserted at `Startup`. Safe.

- [ ] **Step 4: Commit**

```bash
git add player/
git commit -m "feat(shadows): drop shadow under player"
```

---

## Task 8: Wire shadows into NPC + Galen spawns

**Files:**
- Modify: `level/src/npcs.rs`
- Modify: `level/src/galen.rs`

- [ ] **Step 1: NPC shadow**

Edit `/home/ddudson/repos/evergreen/level/src/npcs.rs`. Find `spawn_npc`. Capture `.id()` and call helper.

Add imports:
```rust
use crate::shadows::{spawn_drop_shadow, DropShadowAssets};
use models::shadow::{NPC_SHADOW_HALF_PX, NPC_SHADOW_OFFSET_Y_PX};
```

Modify `spawn_npc` to take `shadow_assets: &DropShadowAssets` parameter. Update the call chain so `spawn_npc_for_area` receives + forwards `Res<DropShadowAssets>`:

```rust
pub fn spawn_npc_for_area(
    commands: &mut Commands,
    asset_server: &AssetServer,
    atlas_layouts: &mut Assets<TextureAtlasLayout>,
    shadow_assets: &DropShadowAssets,
    area: &crate::area::Area,
    area_pos: IVec2,
) {
    let AreaEvent::NpcEncounter(npc_kind) = area.event else {
        return;
    };
    let base = crate::spawning::area_world_offset(area_pos);
    spawn_npc(commands, asset_server, atlas_layouts, shadow_assets, npc_kind, base);
}

fn spawn_npc(
    commands: &mut Commands,
    asset_server: &AssetServer,
    atlas_layouts: &mut Assets<TextureAtlasLayout>,
    shadow_assets: &DropShadowAssets,
    kind: NpcKind,
    base: Vec2,
) {
    let (name, sheet, script, barks) = npc_data(kind);
    let pos = tile_world_pos(PATH_CENTER_X, PATH_CENTER_Y, base);

    let parent = commands.spawn((
        EventNpc,
        Name::new(name),
        npc_sprite(asset_server, atlas_layouts, sheet),
        npc_collider(),
        Transform::from_translation(pos),
        npc_anim_bundle(pos.truncate()),
        Talker::new(asset_server.load(script)),
        bark_pool(asset_server, barks),
    )).id();

    spawn_occluder(commands, parent, NPC_BODY_HALF_PX, NPC_BODY_OFFSET_PX);
    spawn_drop_shadow(commands, shadow_assets, parent, NPC_SHADOW_HALF_PX, NPC_SHADOW_OFFSET_Y_PX);
}
```

The caller chain `ensure_area_spawned` must forward the new resource. Read `/home/ddudson/repos/evergreen/level/src/spawning.rs` to find where `spawn_npc_for_area` is called and propagate the `Res<DropShadowAssets>` through. If the chain is `ensure_area_spawned -> spawn_npc_for_area`, then `ensure_area_spawned` itself needs the resource as a parameter -- which means its registered system signature changes. Fully thread the resource through every layer.

- [ ] **Step 2: Galen shadow**

Edit `/home/ddudson/repos/evergreen/level/src/galen.rs`. Locate the `commands.spawn((NpcGalen, ...))` block. Capture `.id()` and call helper.

Add imports:
```rust
use crate::shadows::{spawn_drop_shadow, DropShadowAssets};
use models::shadow::{GALEN_SHADOW_HALF_PX, GALEN_SHADOW_OFFSET_Y_PX};
```

Modify Galen's spawn system signature to include `shadow_assets: Res<DropShadowAssets>`. Capture `.id()` from the existing spawn, then call:

```rust
spawn_drop_shadow(&mut commands, &shadow_assets, parent, GALEN_SHADOW_HALF_PX, GALEN_SHADOW_OFFSET_Y_PX);
```

Verify the existing spawn has `Anchor::default()` (CENTER) -- if BOTTOM_CENTER, the OFFSET_Y_PX of -10 may be wrong. Tune in QA (T12).

- [ ] **Step 3: Build + clippy + tests**

```bash
cargo build
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --target x86_64-unknown-linux-gnu --lib
```

All must pass.

- [ ] **Step 4: Commit**

```bash
git add level/
git commit -m "feat(shadows): drop shadows under NPCs and Galen"
```

---

## Task 9: Wire shadows into tree + creature spawns

**Files:**
- Modify: `level/src/scenery.rs`
- Modify: `level/src/creatures.rs`

- [ ] **Step 1: Tree shadow**

Edit `/home/ddudson/repos/evergreen/level/src/scenery.rs`. Find `spawn_tree`. Already has `.id()` capture (added in Phase 2). Add shadow call after the two occluder spawns.

Add imports:
```rust
use crate::shadows::{spawn_drop_shadow, DropShadowAssets};
use models::shadow::{TREE_SHADOW_HALF_PX, TREE_SHADOW_OFFSET_Y_PX};
```

Modify `spawn_tree` signature to take `shadow_assets: &DropShadowAssets`. Caller chain (`spawn_area_scenery`, `spawn_area_scenery_at`) must forward it. Add `spawn_drop_shadow` call after the existing two `spawn_occluder` calls:

```rust
spawn_occluder(commands, parent, TREE_TRUNK_HALF_PX, TREE_TRUNK_OFFSET_PX);
spawn_occluder(commands, parent, TREE_CANOPY_HALF_PX, TREE_CANOPY_OFFSET_PX);
spawn_drop_shadow(commands, shadow_assets, parent, TREE_SHADOW_HALF_PX, TREE_SHADOW_OFFSET_Y_PX);
```

Update `spawn_area_scenery_at` and `spawn_area_scenery` to take `shadow_assets: &DropShadowAssets` and forward it.

- [ ] **Step 2: Creature shadow**

Edit `/home/ddudson/repos/evergreen/level/src/creatures.rs`. Find the spawn site (line ~249). Capture `.id()` and call helper.

Add imports:
```rust
use crate::shadows::{spawn_drop_shadow, DropShadowAssets};
use models::shadow::{CREATURE_SHADOW_HALF_PX, CREATURE_SHADOW_OFFSET_Y_PX};
```

Modify the spawn function (likely `spawn_area_creatures` or similar -- read the file to confirm name) to take `shadow_assets: &DropShadowAssets`. Replace:

```rust
commands.spawn((
    Creature,
    CreatureAi::new(def.speed, def.movement, entity_seed),
    Sprite { ... },
    Transform::from_xyz(world_x, world_y, z),
));
```

with:

```rust
let parent = commands.spawn((
    Creature,
    CreatureAi::new(def.speed, def.movement, entity_seed),
    Sprite { ... },
    Transform::from_xyz(world_x, world_y, z),
)).id();

spawn_drop_shadow(commands, shadow_assets, parent, CREATURE_SHADOW_HALF_PX, CREATURE_SHADOW_OFFSET_Y_PX);
```

Forward the resource through the spawn caller chain (likely `ensure_area_spawned` or similar).

- [ ] **Step 3: Build + clippy + tests**

```bash
cargo build
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --target x86_64-unknown-linux-gnu --lib
```

All must pass.

- [ ] **Step 4: Commit**

```bash
git add level/
git commit -m "feat(shadows): drop shadows under trees and creatures"
```

---

## Task 10: Wire shadow into grass spawn (perf risk)

**Files:**
- Modify: `level/src/grass.rs`

- [ ] **Step 1: Grass shadow**

Edit `/home/ddudson/repos/evergreen/level/src/grass.rs`. Find `spawn_area_grass`. Already has `.id()` capture (added in Phase 2 for occluders). Add shadow call after the existing `spawn_occluder` call.

Add imports:
```rust
use crate::shadows::{spawn_drop_shadow, DropShadowAssets};
use models::shadow::{GRASS_SHADOW_HALF_PX, GRASS_SHADOW_OFFSET_Y_PX};
```

Modify `spawn_area_grass` to take `shadow_assets: &DropShadowAssets`. Add the call after the existing occluder spawn:

```rust
spawn_occluder(commands, parent, GRASS_OCCLUDER_HALF_PX, GRASS_OCCLUDER_OFFSET_PX);
spawn_drop_shadow(commands, shadow_assets, parent, GRASS_SHADOW_HALF_PX, GRASS_SHADOW_OFFSET_Y_PX);
```

Forward the resource through the caller chain.

- [ ] **Step 2: Build + clippy + tests**

```bash
cargo build
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --target x86_64-unknown-linux-gnu --lib
```

All must pass.

- [ ] **Step 3: WASM perf check**

Run: `trunk serve`. Open `http://127.0.0.1:8080`. Walk through a grass-heavy greenwood area. Open DevTools → Performance, record 5 seconds.

Expected: frame time < 16.6 ms.

If FPS drops, revert this commit only:
```bash
git revert HEAD
```

Trees + NPCs + creatures + Galen + player still get shadows.

- [ ] **Step 4: Commit**

```bash
git add level/src/grass.rs
git commit -m "feat(shadows): drop shadows under grass tufts"
```

---

## Task 11: Bayer 4x4 dither in `BiomeAtmosphere` shader

Goal: hide color banding in dark biomes via ordered dither, scaled by `darkness`.

**Files:**
- Modify: `assets/shaders/biome_atmosphere.wgsl`

- [ ] **Step 1: Edit shader**

Replace the entire content of `/home/ddudson/repos/evergreen/assets/shaders/biome_atmosphere.wgsl` with:

```wgsl
#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput

@group(0) @binding(0) var screen_texture: texture_2d<f32>;
@group(0) @binding(1) var texture_sampler: sampler;

struct BiomeAtmosphere {
    // Biome darkness: 0.0 = city (bright), 1.0 = darkwood (dark)
    darkness: f32,
    _pad0: f32,
    _pad1: f32,
    _pad2: f32,
}

@group(0) @binding(2) var<uniform> settings: BiomeAtmosphere;

// 4x4 Bayer ordered-dither matrix, normalized to [0, 1).
fn bayer_4x4(coord: vec2<u32>) -> f32 {
    let i = (coord.y % 4u) * 4u + (coord.x % 4u);
    var m: array<u32, 16> = array<u32, 16>(
        0u,  8u,  2u,  10u,
        12u, 4u,  14u, 6u,
        3u,  11u, 1u,  9u,
        15u, 7u,  13u, 5u,
    );
    return f32(m[i]) / 16.0;
}

const DITHER_STEP: f32 = 1.0 / 255.0;

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    let color = textureSample(screen_texture, texture_sampler, in.uv);

    // -- Biome darkening --
    let darken = 1.0 - settings.darkness * 0.80;

    // -- Vignette --
    let center = in.uv - vec2<f32>(0.5, 0.5);
    let dist = length(center);
    let vignette_radius = 0.15;
    let vignette_soft = 0.35;
    let raw_vignette = smoothstep(vignette_radius, vignette_radius + vignette_soft, dist);
    let vignette = 1.0 - raw_vignette * settings.darkness;

    var rgb = color.rgb * darken * vignette;

    // -- Bayer dither, scaled by darkness so city = no dither, darkwood = full --
    let frag_coord = vec2<u32>(in.position.xy);
    let dither = (bayer_4x4(frag_coord) - 0.5) * DITHER_STEP * settings.darkness;
    rgb = rgb + vec3<f32>(dither, dither, dither);

    return vec4<f32>(rgb, color.a);
}
```

- [ ] **Step 2: Build (shader compiles via Bevy at runtime)**

```bash
cargo build
```

The shader is loaded at runtime; build success only confirms Rust compiles. The shader itself is validated by wgpu when first used.

- [ ] **Step 3: Quick visual check via `trunk serve`**

Run `trunk serve`. Walk into a darkwood area at night. The previously-visible banding in dark blue ambient should now show stippled dither pattern instead of smooth bands.

If the shader fails to compile at runtime, the WebGL console will show a wgsl error. Common pitfalls:
- `var m: array<u32, 16>` syntax may need `let` in some wgpu versions -- if so, change to `let m = array<u32, 16>(...)` and adjust.
- `vec2<u32>(in.position.xy)` requires `in.position` to exist on `FullscreenVertexOutput`. If not, use `in.uv * vec2<f32>(textureDimensions(screen_texture))` cast to u32.

- [ ] **Step 4: Commit**

```bash
git add assets/shaders/biome_atmosphere.wgsl
git commit -m "feat(shader): Bayer 4x4 ordered dither hides darkwood banding"
```

---

## Task 12: Final QA + cleanup

**Files:** none (manual + optional tuning commits)

- [ ] **Step 1: Workspace verification**

Run from `/home/ddudson/repos/evergreen`:
- `cargo build --release`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace --target x86_64-unknown-linux-gnu --lib`
- `cargo fmt -- --check`

All must pass. Pre-existing keybinds doctest failure OUT OF SCOPE.

- [ ] **Step 2: Visual smoke test (`trunk serve`)**

Walk through state combinations:

- **City @ midday/clear**: shadows visible under player + scenery; rare dust motes.
- **Greenwood @ afternoon**: dust motes drift; shadows visible.
- **Darkwood @ midday**: fog drifts; no fireflies; shadows visible.
- **Darkwood @ night**: fireflies pulse-bloom; fog persists; ambient blue but darkwood banding hidden by dither.
- **Trigger Rain weather** (wait or use a debug shortcut if exists): dust motes stop.
- **Pause/Dialogue**: weather particles freeze; shadows persist as static children.
- **MainMenu**: ambient resets to neutral (Phase 2 fix); no particles; no shadows (no entities spawned).
- **GameOver**: same as MainMenu.

- [ ] **Step 3: Shadow Z-order check**

Walk player past trees in different y positions. Player shadow should:
- Be behind player sprite (rendered first).
- Not flicker over/under tree shadows when overlapping.

If shadows render on top of sprites (Z-order wrong), `SHADOW_Z_OFFSET = -0.1` in `level/src/shadows.rs` may need adjustment. Tune and commit:

```bash
git add level/src/shadows.rs
git commit -m "tune(shadows): adjust SHADOW_Z_OFFSET for correct render order"
```

- [ ] **Step 4: Firefly bloom check**

In darkwood @ night, fireflies should glow with bloom halo (Phase 1 bloom + Phase 3 emissive FIREFLY color >1.0). If not visible, check:
- Bloom threshold in `post_processing/src/bloom_setup.rs` (set at 1.0 in Phase 1) -- still effective?
- FIREFLY constant magnitude (R=2.0, G=3.0) -- minimum needs to exceed 1.0 after multiplication with night ambient (0.30 brightness). 2.0 * 0.3 = 0.6 -- BELOW threshold. The pulse-flicker animation drops alpha, not color, so RGB stays at 2.0/3.0/0.5 in linear.

Wait -- ambient is multiplicative on RGB, so firefly color becomes 2.0*0.3 = 0.6, sub-bloom-threshold. To compensate, raise FIREFLY constants:

If fireflies don't bloom:
- Edit `models/src/palette.rs`: `pub const FIREFLY: Color = Color::srgb(8.0, 12.0, 2.0);`
- Commit:
  ```bash
  git add models/src/palette.rs
  git commit -m "tune(palette): boost FIREFLY emissive to bloom through night ambient"
  ```

- [ ] **Step 5: Fog visibility check**

Darkwood fog should be visible but not opaque-blocking. Alpha `0.35` should let player + sprites through. If too thick, reduce to `0.20` in palette. If invisible, raise to `0.50`.

- [ ] **Step 6: Final summary commit (only if tunings were applied)**

If no tunings, no commit. Otherwise the tuning commits land in Step 3-5 above.

---

## Self-Review Checklist (already applied by author)

**1. Spec coverage:**

| Spec section | Task |
|---|---|
| New ParticleVariant enum members | T1 |
| Palette constants (FIREFLY, DUST_MOTE, FOG, DROP_SHADOW) | T1 |
| Shadow geometry constants in models::shadow | T1 |
| Firefly predicate + spawning + tests | T2 |
| Firefly pulse animation | T3 |
| Dust mote predicate + spawning + tests | T4 |
| Fog predicate + spawning + tests | T5 |
| DropShadowAssets resource + helper + startup | T6 |
| Player drop shadow | T7 |
| NPC + Galen drop shadows | T8 |
| Tree + creature drop shadows | T9 |
| Grass drop shadow (perf risk last) | T10 |
| Bayer dither in shader | T11 |
| Visual + perf QA | T12 |

No gaps.

**2. Placeholder scan:**
- T8 says "Verify the existing spawn has `Anchor::default()` (CENTER)... If BOTTOM_CENTER, the OFFSET_Y_PX of -10 may be wrong" -- this is a verify-and-tune flag, not a placeholder. Acceptable.
- T11 step 3 lists wgsl pitfalls with concrete fixes -- not "TBD".
- T12 has tuning commits explicitly noted as conditional ("only if X"). Each names exact files + messages.

**3. Type consistency:**
- `AreaAlignment` (u8) used in `firefly_active` and `fog_active` -- consistent.
- `WeatherKind` enum used in `dust_mote_active` matches existing `models::weather::WeatherKind`.
- `DropShadowAssets` referenced as `Res<DropShadowAssets>` in spawn systems and `&DropShadowAssets` in helpers -- consistent.
- `spawn_drop_shadow(commands, assets, parent, half_size, offset_y)` signature identical across all 6 spawn-site call sites (T7-T10).
- `ChildOf(parent)` used in `spawn_drop_shadow` matches existing Phase 2 occluder pattern.
