# HDR + Bloom + Per-Biome Color Grading Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add HDR rendering, Bloom post-processing, and per-biome `ColorGrading` (LUT-style mood shift) to the existing 2D camera so emissive sprites pop and each biome has a distinct cinematic feel.

**Architecture:** Enable HDR on `Camera2d` and pair it with `Tonemapping::TonyMcMapface` + `DebandDither::Enabled`. Add a low-intensity `Bloom` component tuned for pixel art (high threshold, modest intensity). Add a `ColorGrading` component on the camera and a system in the `post_processing` crate that lerps `ColorGradingGlobal` fields (exposure, temperature, tint, post_saturation) toward biome-specific targets driven by the active area's alignment (1=city, 50=greenwood, 100=darkwood). The existing `BiomeAtmosphere` fullscreen shader pass continues to run after tonemapping — unchanged. Color grading is a stylistic accent layered on top.

**Tech Stack:** Bevy 0.18.1, Rust nightly, WASM (wgpu/WebGL2). Existing crates: `post_processing`, `camera`, `level`, `models`.

---

## File Structure

**Create:**
- `post_processing/src/bloom_setup.rs` — Bloom preset constants (intensity, threshold, etc.)
- `post_processing/src/grading.rs` — `BiomeGradingTargets` struct, alignment→`ColorGradingGlobal` interpolation, `sync_color_grading` system
- `post_processing/src/grading_test.rs` — pure-function tests for biome target lookup and lerp

**Modify:**
- `camera/src/plugin.rs` — add HDR, `Tonemapping`, `DebandDither`, `Bloom`, `ColorGrading` to `Camera2d` spawn
- `post_processing/src/plugin.rs` — register `sync_color_grading` system
- `post_processing/src/lib.rs` — export `bloom_setup`, `grading`
- `post_processing/Cargo.toml` — no changes (Bevy already a dep; bloom + grading are in `bevy::post_process` and `bevy::render::view`)
- `models/src/palette.rs` — no changes (grading uses temperature/tint as f32 stops, not `Color` constants)

---

## Task 1: Enable HDR + Tonemapping + DebandDither on Camera2d

Goal: turn on the high-dynamic-range pipeline on the camera. Without HDR, `Bloom` is a no-op. `Tonemapping::TonyMcMapface` is the recommended companion (desaturates highlights cleanly). `DebandDither::Enabled` removes banding introduced by the bloom blur.

**Files:**
- Modify: `camera/src/plugin.rs`

- [ ] **Step 1: Open `camera/src/plugin.rs` and add imports**

Bevy 0.18 removed `Camera { hdr: true }`. HDR is now enabled via the `Hdr` marker component from `bevy::render::view`.

Add to the top of the file (alongside existing `bevy::prelude::*`):

```rust
use bevy::core_pipeline::tonemapping::{DebandDither, Tonemapping};
use bevy::render::view::Hdr;
```

- [ ] **Step 2: Update `Camera2d` spawn to enable HDR**

Locate the `setup` function in `camera/src/plugin.rs:33-45`. Replace the `commands.spawn((...))` block with:

```rust
commands.spawn((
    Camera2d,
    Hdr,
    Tonemapping::TonyMcMapface,
    DebandDither::Enabled,
    BiomeAtmosphere::default(),
    Projection::Orthographic(OrthographicProjection {
        scaling_mode: ScalingMode::AutoMin {
            min_width: f32::from(MAP_WIDTH) * f32::from(TILE_SIZE_PX),
            min_height: f32::from(MAP_HEIGHT) * f32::from(TILE_SIZE_PX),
        },
        ..OrthographicProjection::default_2d()
    }),
));
```

- [ ] **Step 3: Verify build passes**

Run: `cargo build`
Expected: build succeeds with no errors. (Warnings about unused imports if `Tonemapping`/`DebandDither` were previously imported elsewhere — fix those.)

- [ ] **Step 4: Run clippy**

Run: `cargo clippy --workspace --all-targets -- -D warnings`
Expected: no clippy errors.

- [ ] **Step 5: Visual smoke test**

Run: `trunk serve`
Open `http://127.0.0.1:8080`. Confirm the game still renders. Colors may look subtly different (tonemapping curve), but no black screen or broken sprites. Existing `BiomeAtmosphere` darkening + ToD tint must still work.

- [ ] **Step 6: Commit**

```bash
git add camera/src/plugin.rs
git commit -m "feat(camera): enable HDR + TonyMcMapface tonemapping + deband dither"
```

---

## Task 2: Add Bloom preset module

Goal: define a single named constant for the Bloom configuration tuned for pixel art (high threshold so only true highlights bloom, low intensity to avoid muddying sharp pixels).

**Files:**
- Create: `post_processing/src/bloom_setup.rs`
- Modify: `post_processing/src/lib.rs`

- [ ] **Step 1: Create `post_processing/src/bloom_setup.rs`**

Write this exact content:

```rust
use bevy::math::Vec2;
use bevy::post_process::bloom::{Bloom, BloomCompositeMode, BloomPrefilter};

/// Bloom intensity tuned low for pixel art -- only true highlights halo.
const BLOOM_INTENSITY: f32 = 0.15;
/// Low-frequency bloom slice contribution.
const BLOOM_LOW_FREQUENCY_BOOST: f32 = 0.7;
/// Curvature of the boost falloff.
const BLOOM_LOW_FREQUENCY_BOOST_CURVATURE: f32 = 0.95;
/// Highpass filter response (1.0 = no highpass; lower preserves more haze).
const BLOOM_HIGH_PASS_FREQUENCY: f32 = 1.0;
/// Threshold (in HDR linear units) above which a pixel contributes to bloom.
/// Set at 1.0 so only emissive (>1.0) sprites trigger; standard sprites stay sharp.
const BLOOM_PREFILTER_THRESHOLD: f32 = 1.0;
/// Soft knee around the threshold for smoother transitions.
const BLOOM_PREFILTER_THRESHOLD_SOFTNESS: f32 = 0.4;
/// Maximum dimension of the bloom mip chain, in pixels. Matches Bevy's default.
const BLOOM_MAX_MIP_DIMENSION_PX: u32 = 512;

/// Returns a `Bloom` component tuned for the project's pixel-art look.
pub fn pixel_art_bloom() -> Bloom {
    Bloom {
        intensity: BLOOM_INTENSITY,
        low_frequency_boost: BLOOM_LOW_FREQUENCY_BOOST,
        low_frequency_boost_curvature: BLOOM_LOW_FREQUENCY_BOOST_CURVATURE,
        high_pass_frequency: BLOOM_HIGH_PASS_FREQUENCY,
        prefilter: BloomPrefilter {
            threshold: BLOOM_PREFILTER_THRESHOLD,
            threshold_softness: BLOOM_PREFILTER_THRESHOLD_SOFTNESS,
        },
        composite_mode: BloomCompositeMode::Additive,
        max_mip_dimension: BLOOM_MAX_MIP_DIMENSION_PX,
        scale: Vec2::ONE,
    }
}
```

- [ ] **Step 2: Export the new module**

Edit `post_processing/src/lib.rs`. Replace the file contents with:

```rust
pub mod atmosphere;
pub mod bloom_setup;
pub mod plugin;
mod sync;
pub mod time_sync;
```

- [ ] **Step 3: Verify build**

Run: `cargo build -p post_processing`
Expected: build succeeds.

If the build fails because `BloomPrefilter`, `BloomCompositeMode`, or `Bloom` field names differ from the snippet above, run:

```bash
cargo doc -p bevy --no-deps --open
```

and search for `Bloom` in the docs (under `bevy::post_process::bloom`) to confirm the exact field names for 0.18.1, then update `bloom_setup.rs` accordingly.

- [ ] **Step 4: Commit**

```bash
git add post_processing/src/bloom_setup.rs post_processing/src/lib.rs
git commit -m "feat(post_processing): add pixel-art bloom preset"
```

---

## Task 3: Attach Bloom to the camera

Goal: spawn the camera with the Bloom component so HDR brights actually halo.

**Files:**
- Modify: `camera/src/plugin.rs`
- Modify: `camera/Cargo.toml` (no changes expected — `post_processing` is already a dep)

- [ ] **Step 1: Add the import**

In `camera/src/plugin.rs`, add to the `use post_processing::...` block:

```rust
use post_processing::atmosphere::BiomeAtmosphere;
use post_processing::bloom_setup::pixel_art_bloom;
```

- [ ] **Step 2: Insert `Bloom` into the camera spawn**

Add `pixel_art_bloom(),` to the spawn tuple, between `DebandDither::Enabled,` and `BiomeAtmosphere::default(),`:

```rust
commands.spawn((
    Camera2d,
    Hdr,
    Tonemapping::TonyMcMapface,
    DebandDither::Enabled,
    pixel_art_bloom(),
    BiomeAtmosphere::default(),
    Projection::Orthographic(OrthographicProjection {
        scaling_mode: ScalingMode::AutoMin {
            min_width: f32::from(MAP_WIDTH) * f32::from(TILE_SIZE_PX),
            min_height: f32::from(MAP_HEIGHT) * f32::from(TILE_SIZE_PX),
        },
        ..OrthographicProjection::default_2d()
    }),
));
```

- [ ] **Step 3: Verify build + clippy**

Run in parallel:
- `cargo build`
- `cargo clippy --workspace --all-targets -- -D warnings`

Expected: both pass.

- [ ] **Step 4: Visual smoke test**

Run: `trunk serve`
Confirm the game runs. Standard sprites (color ≤ 1.0) should look the same. There is no emissive sprite in the project yet, so bloom is a no-op visually until Task 6 adds a test fixture.

- [ ] **Step 5: Commit**

```bash
git add camera/src/plugin.rs
git commit -m "feat(camera): attach pixel-art bloom to Camera2d"
```

---

## Task 4: Define biome grading targets (pure functions, TDD)

Goal: a small data-only module that maps an alignment value (1-100) to a `ColorGradingGlobal` target, plus a lerp helper. Tested with no Bevy app — pure math.

**Files:**
- Create: `post_processing/src/grading.rs`
- Modify: `post_processing/src/lib.rs`

- [ ] **Step 1: Write the failing test**

Create `post_processing/src/grading.rs` with this content (tests live in the same file via `#[cfg(test)] mod tests`):

```rust
use bevy::prelude::*;
use bevy::render::view::{ColorGrading, ColorGradingGlobal};

/// Alignment scale: 1 = full city, 50 = greenwood, 100 = full darkwood.
type AreaAlignment = u8;

/// Anchor alignment for the city biome.
const ALIGNMENT_CITY: f32 = 1.0;
/// Anchor alignment for the greenwood biome.
const ALIGNMENT_GREENWOOD: f32 = 50.0;
/// Anchor alignment for the darkwood biome.
const ALIGNMENT_DARKWOOD: f32 = 100.0;

/// City: warm afternoon stone, slight desaturation.
const CITY_EXPOSURE: f32 = 0.05;
const CITY_TEMPERATURE: f32 = 0.18;
const CITY_TINT: f32 = -0.05;
const CITY_POST_SATURATION: f32 = 0.92;

/// Greenwood: vivid, neutral white balance.
const GREENWOOD_EXPOSURE: f32 = 0.0;
const GREENWOOD_TEMPERATURE: f32 = -0.02;
const GREENWOOD_TINT: f32 = 0.05;
const GREENWOOD_POST_SATURATION: f32 = 1.10;

/// Darkwood: cool, slightly underexposed, muted.
const DARKWOOD_EXPOSURE: f32 = -0.20;
const DARKWOOD_TEMPERATURE: f32 = -0.30;
const DARKWOOD_TINT: f32 = 0.10;
const DARKWOOD_POST_SATURATION: f32 = 0.85;

/// One biome's target grading values.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BiomeGradingTarget {
    pub exposure: f32,
    pub temperature: f32,
    pub tint: f32,
    pub post_saturation: f32,
}

impl BiomeGradingTarget {
    const CITY: Self = Self {
        exposure: CITY_EXPOSURE,
        temperature: CITY_TEMPERATURE,
        tint: CITY_TINT,
        post_saturation: CITY_POST_SATURATION,
    };
    const GREENWOOD: Self = Self {
        exposure: GREENWOOD_EXPOSURE,
        temperature: GREENWOOD_TEMPERATURE,
        tint: GREENWOOD_TINT,
        post_saturation: GREENWOOD_POST_SATURATION,
    };
    const DARKWOOD: Self = Self {
        exposure: DARKWOOD_EXPOSURE,
        temperature: DARKWOOD_TEMPERATURE,
        tint: DARKWOOD_TINT,
        post_saturation: DARKWOOD_POST_SATURATION,
    };

    fn lerp(self, other: Self, t: f32) -> Self {
        Self {
            exposure: lerp(self.exposure, other.exposure, t),
            temperature: lerp(self.temperature, other.temperature, t),
            tint: lerp(self.tint, other.tint, t),
            post_saturation: lerp(self.post_saturation, other.post_saturation, t),
        }
    }
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

/// Map an alignment value to a target grading by interpolating between anchors.
pub fn target_for_alignment(alignment: AreaAlignment) -> BiomeGradingTarget {
    let a = f32::from(alignment.clamp(1, 100));
    if a <= ALIGNMENT_GREENWOOD {
        let t = (a - ALIGNMENT_CITY) / (ALIGNMENT_GREENWOOD - ALIGNMENT_CITY);
        BiomeGradingTarget::CITY.lerp(BiomeGradingTarget::GREENWOOD, t)
    } else {
        let t = (a - ALIGNMENT_GREENWOOD) / (ALIGNMENT_DARKWOOD - ALIGNMENT_GREENWOOD);
        BiomeGradingTarget::GREENWOOD.lerp(BiomeGradingTarget::DARKWOOD, t)
    }
}

/// Apply a `BiomeGradingTarget` to a `ColorGrading` component (writes the
/// `global` section, leaves shadows/midtones/highlights untouched).
pub fn apply_target(grading: &mut ColorGrading, target: BiomeGradingTarget) {
    grading.global = ColorGradingGlobal {
        exposure: target.exposure,
        temperature: target.temperature,
        tint: target.tint,
        post_saturation: target.post_saturation,
        ..grading.global.clone()
    };
}

/// Lerp speed for color grading transitions between areas (per second).
pub const GRADING_LERP_SPEED: f32 = 2.5;

/// Lerp current grading toward target by `alpha` (0..1, single-frame step).
pub fn step_toward(current: BiomeGradingTarget, target: BiomeGradingTarget, alpha: f32) -> BiomeGradingTarget {
    current.lerp(target, alpha.clamp(0.0, 1.0))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn approx(a: f32, b: f32) {
        assert!((a - b).abs() < 1e-4, "expected ~{b}, got {a}");
    }

    #[test]
    fn target_at_city_anchor_returns_city() {
        let t = target_for_alignment(1);
        approx(t.exposure, CITY_EXPOSURE);
        approx(t.temperature, CITY_TEMPERATURE);
        approx(t.tint, CITY_TINT);
        approx(t.post_saturation, CITY_POST_SATURATION);
    }

    #[test]
    fn target_at_greenwood_anchor_returns_greenwood() {
        let t = target_for_alignment(50);
        approx(t.exposure, GREENWOOD_EXPOSURE);
        approx(t.temperature, GREENWOOD_TEMPERATURE);
        approx(t.tint, GREENWOOD_TINT);
        approx(t.post_saturation, GREENWOOD_POST_SATURATION);
    }

    #[test]
    fn target_at_darkwood_anchor_returns_darkwood() {
        let t = target_for_alignment(100);
        approx(t.exposure, DARKWOOD_EXPOSURE);
        approx(t.temperature, DARKWOOD_TEMPERATURE);
        approx(t.tint, DARKWOOD_TINT);
        approx(t.post_saturation, DARKWOOD_POST_SATURATION);
    }

    #[test]
    fn target_midway_city_greenwood_is_average() {
        let t = target_for_alignment(25); // halfway-ish (between 1 and 50)
        let expected_exposure = lerp(CITY_EXPOSURE, GREENWOOD_EXPOSURE, (25.0 - 1.0) / 49.0);
        approx(t.exposure, expected_exposure);
    }

    #[test]
    fn target_clamps_below_one() {
        let t = target_for_alignment(0);
        approx(t.exposure, CITY_EXPOSURE);
    }

    #[test]
    fn step_toward_zero_alpha_returns_current() {
        let current = BiomeGradingTarget::CITY;
        let target = BiomeGradingTarget::DARKWOOD;
        let result = step_toward(current, target, 0.0);
        approx(result.exposure, current.exposure);
    }

    #[test]
    fn step_toward_one_alpha_returns_target() {
        let current = BiomeGradingTarget::CITY;
        let target = BiomeGradingTarget::DARKWOOD;
        let result = step_toward(current, target, 1.0);
        approx(result.exposure, target.exposure);
    }
}
```

- [ ] **Step 2: Register the module**

Edit `post_processing/src/lib.rs`:

```rust
pub mod atmosphere;
pub mod bloom_setup;
pub mod grading;
pub mod plugin;
mod sync;
pub mod time_sync;
```

- [ ] **Step 3: Run tests, expect them to pass**

Run: `cargo test -p post_processing grading`
Expected: 7 tests pass.

If any test fails because `ColorGrading::global` field name differs, run `cargo doc -p bevy --no-deps --open`, search for `ColorGrading`, fix the snippet, and re-run.

- [ ] **Step 4: Run clippy**

Run: `cargo clippy -p post_processing --all-targets -- -D warnings`
Expected: no errors.

- [ ] **Step 5: Commit**

```bash
git add post_processing/src/grading.rs post_processing/src/lib.rs
git commit -m "feat(post_processing): biome color grading targets + lerp helpers"
```

---

## Task 5: Wire `ColorGrading` to camera and add sync system

Goal: spawn the `ColorGrading` component on the camera and run a system every frame that interpolates the grading toward the current area's target, mirroring how `sync_atmosphere` works.

**Files:**
- Modify: `camera/src/plugin.rs`
- Modify: `post_processing/src/grading.rs` — add the system function
- Modify: `post_processing/src/plugin.rs` — register the system

- [ ] **Step 1: Append the sync system to `grading.rs`**

At the bottom of `post_processing/src/grading.rs` (above `#[cfg(test)] mod tests`), add:

```rust
use level::world::WorldMap;

/// Per-frame system: read current area's alignment from `WorldMap`,
/// compute the target `BiomeGradingTarget`, lerp the camera's grading toward it.
pub fn sync_color_grading(
    world: Res<WorldMap>,
    time: Res<Time>,
    mut query: Query<&mut ColorGrading>,
) {
    let alignment = world.get_area(world.current).map_or(50, |a| a.alignment);
    let target = target_for_alignment(alignment);
    let alpha = (GRADING_LERP_SPEED * time.delta_secs()).min(1.0);

    for mut grading in &mut query {
        let current = BiomeGradingTarget {
            exposure: grading.global.exposure,
            temperature: grading.global.temperature,
            tint: grading.global.tint,
            post_saturation: grading.global.post_saturation,
        };
        let next = step_toward(current, target, alpha);
        apply_target(&mut grading, next);
    }
}
```

- [ ] **Step 2: Register the system in `post_processing/src/plugin.rs`**

Replace the `add_systems(Update, ...)` call in `post_processing/src/plugin.rs:18-22` with one that includes `sync_color_grading`:

```rust
use crate::grading::sync_color_grading;
```

Add the import at the top with the other `use crate::*` lines, then update the `Update` block:

```rust
app.add_systems(
    Update,
    (
        sync::sync_atmosphere,
        time_sync::tick_game_clock,
        sync_color_grading,
    )
        .run_if(in_state(GameState::Playing)),
);
```

- [ ] **Step 3: Attach `ColorGrading` to the camera spawn**

In `camera/src/plugin.rs`, add the import:

```rust
use bevy::render::view::ColorGrading;
```

Add `ColorGrading::default(),` to the spawn tuple, after `BiomeAtmosphere::default(),`:

```rust
commands.spawn((
    Camera2d,
    Hdr,
    Tonemapping::TonyMcMapface,
    DebandDither::Enabled,
    pixel_art_bloom(),
    BiomeAtmosphere::default(),
    ColorGrading::default(),
    Projection::Orthographic(OrthographicProjection {
        scaling_mode: ScalingMode::AutoMin {
            min_width: f32::from(MAP_WIDTH) * f32::from(TILE_SIZE_PX),
            min_height: f32::from(MAP_HEIGHT) * f32::from(TILE_SIZE_PX),
        },
        ..OrthographicProjection::default_2d()
    }),
));
```

- [ ] **Step 4: Build + clippy + tests**

Run in parallel:
- `cargo build`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace`

Expected: all pass. If `level::world::WorldMap` is not directly accessible from `post_processing`, verify `Cargo.toml` for `post_processing` already lists `level = { path = "../level" }` (it does, per the existing `sync.rs`).

- [ ] **Step 5: Visual smoke test — biome transitions**

Run: `trunk serve`. Move the player across area boundaries (city → greenwood → darkwood). Confirm:
- City areas look slightly warm + desaturated.
- Greenwood looks vivid/saturated.
- Darkwood looks cool/blue-tinted, dimmer.
- Transitions blend smoothly (~0.4s) rather than snapping.

- [ ] **Step 6: Commit**

```bash
git add camera/src/plugin.rs post_processing/src/grading.rs post_processing/src/plugin.rs
git commit -m "feat(post_processing): sync ColorGrading to area alignment per biome"
```

---

## Task 6: Add a test emissive sprite to verify bloom

Goal: drop a single bright (>1.0) sprite into the scene as a visual verification fixture. Without an emissive sprite, bloom has nothing to act on and the visual change from Task 3 is invisible. After confirming bloom works, decide whether to keep the sprite or revert it.

**Files:**
- Modify: `level/src/decorations.rs` (or wherever scenery sprites are spawned — verify in step 1)

- [ ] **Step 1: Find the scenery spawn site**

Run: `cargo doc -p level --no-deps --open` is overkill; instead grep:

Use the Grep tool with pattern `Sprite \{` in the `level/` directory.
Identify the spawn function for trees, lanterns, or any visible decoration. The first scenery spawn that always renders is the right target.

- [ ] **Step 2: Bump one sprite's color above 1.0**

Pick a single scenery item (e.g. one specific tile). Change its `Sprite::color` to `Color::srgb(2.5, 2.0, 1.5)` (warm bright glow). Use the existing palette lookup pattern; if no `Color::srgb` is allowed (palette lint), add a new `LANTERN_GLOW` constant to `models/src/palette.rs`:

```rust
#[allow(clippy::disallowed_methods)]
pub const LANTERN_GLOW: Color = Color::srgb(2.5, 2.0, 1.5);
```

Then use `palette::LANTERN_GLOW` at the spawn site.

- [ ] **Step 3: Visual verification**

Run: `trunk serve`. Locate the modified sprite in-game. Confirm a soft halo / glow surrounds it (bloom). Without bloom: hard edges only. With bloom: warm halo extending ~10-20 pixels past the sprite outline.

- [ ] **Step 4: Decide — keep or revert**

If the glowing sprite makes sense in-world (e.g. you picked a lantern), keep it. If you picked a random tree as a test fixture, revert that one sprite back to its original color. Either way:
- `palette::LANTERN_GLOW` constant stays (future emissive sprites will use it).
- The verification confirmed bloom is wired end-to-end.

- [ ] **Step 5: Final build + clippy + test sweep**

Run in parallel:
- `cargo build --release`  (catches release-only optimizations issues)
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace`

Expected: all pass.

- [ ] **Step 6: Commit**

```bash
git add models/src/palette.rs level/src/<modified file>
git commit -m "feat(palette): add LANTERN_GLOW emissive color for HDR bloom"
```

---

## Task 7: Final manual QA pass

Goal: confirm the full Phase 1 stack — HDR + Bloom + per-biome ColorGrading — works together without regressions in existing systems (atmosphere darkness, time-of-day tint).

**Files:** none (manual test only).

- [ ] **Step 1: Test all GameStates**

Run: `trunk serve`. Walk through each state:
- MainMenu → confirm it renders normally (camera spawns at startup, so HDR is on even in menus — UI text/buttons should be unaffected).
- Playing → confirm gameplay renders, biome grading shifts as you cross areas.
- Pause → confirm freeze behaves correctly (atmosphere/grading systems gate on `Playing`, so they pause).
- Dialogue → confirm world-frozen rendering still works.
- LorePage / KeybindConfig → confirm UI screens render correctly over the HDR pipeline.

- [ ] **Step 2: Test time-of-day at full range**

If a debug time-skip exists (check `diagnostics/src/overlay.rs`), advance the clock through dawn/midday/dusk/night. Confirm:
- ToD tint from `BiomeAtmosphere` still applies on top of `ColorGrading`.
- No double-darkening or washed-out look at night.
- If the layered effect is too dim at night in a darkwood area, note it as a Phase 2 tuning task — do not block Phase 1.

- [ ] **Step 3: WASM perf check**

Open DevTools → Performance tab. Record 5 seconds of gameplay. Confirm frame time stays under 16.6ms (60 FPS). If bloom pushes WASM over budget, lower `max_mip_dimension` in `bloom_setup.rs` from 512 to 256.

- [ ] **Step 4: Fmt check**

Run: `cargo fmt -- --check`
Expected: clean.

- [ ] **Step 5: Final commit (if any cleanup)**

If Step 3 required tuning, commit:

```bash
git add post_processing/src/bloom_setup.rs
git commit -m "perf(bloom): lower max_mip_dimension for WASM"
```

Otherwise no commit needed.

---

## Self-Review Checklist (already applied by author)

- **Spec coverage:** HDR ✓ (Task 1), Bloom ✓ (Tasks 2-3), per-biome LUT-style grading ✓ (Tasks 4-5), verification fixture ✓ (Task 6), QA ✓ (Task 7).
- **Placeholder scan:** No "TBD"/"add appropriate"/"similar to". Two intentional escape hatches (Task 2 Step 3 + Task 4 Step 3) tell the engineer how to verify Bevy 0.18.1 API names if a field rename slipped through — these are safety nets, not placeholders.
- **Type consistency:** `BiomeGradingTarget` fields used identically across `target_for_alignment`, `step_toward`, `apply_target`, and `sync_color_grading`. `ColorGrading::global` field referenced consistently. `WorldMap::get_area` matches the signature already used in `sync.rs:15`.
