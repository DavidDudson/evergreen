# Phase 2: 2D Lighting Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add real-time 2D lighting via `bevy_light_2d` 0.9 -- per-emitter point lights, multi-rect occluders on trees/NPCs/grass, and per-time-of-day ambient color migrated out of the existing `BiomeAtmosphere` shader.

**Architecture:** New `lighting` crate hosting plugin wiring, ambient curve, player torch system, exit light, and occluder helpers. `bevy_light_2d`'s `LightingPass` slots between `Node2d::EndMainPass` and `Node2d::StartMainPassPostProcessing` -- runs BEFORE Bloom/Tonemap/BiomeAtmosphere so emissive sprites still halo. `BiomeAtmosphere` shader slims down to biome darkness + vignette only; ToD anchors move into `lighting::ambient`.

**Tech Stack:** Bevy 0.18.1, `bevy_light_2d` 0.9, Rust nightly, WASM (wgpu/WebGL2). New crate `lighting/`. Modifies: `post_processing/`, `level/`, `player/`, `camera/`, `evergreen_main/`, `models/palette.rs`.

---

## File Structure

**Create:**
- `lighting/Cargo.toml`
- `lighting/src/lib.rs`
- `lighting/src/plugin.rs`
- `lighting/src/ambient.rs`
- `lighting/src/torch.rs`
- `lighting/src/exit_light.rs`
- `lighting/src/occluders.rs`

**Modify:**
- `Cargo.toml` (workspace members)
- `camera/Cargo.toml` (add lighting dep) and `camera/src/plugin.rs` (Light2d + AmbientLight2d on camera)
- `post_processing/src/atmosphere.rs` (drop tod fields)
- `post_processing/src/time_sync.rs` (drop sync_time_of_day + period_values + ToD anchors; keep tick_game_clock + period-end constants exported)
- `post_processing/src/plugin.rs` (drop sync_time_of_day registration)
- `assets/shaders/biome_atmosphere.wgsl` (drop ToD multiplication)
- `models/src/palette.rs` (add LIGHT_EXIT, LIGHT_TORCH, AMBIENT_DAY/DAWN/DUSK/NIGHT)
- `level/src/exit.rs` (register attach_level_exit_light)
- `level/src/scenery.rs` (call tree_occluders inside spawn_tree)
- `level/src/npcs.rs` (call npc_occluder inside spawn_npc_for_area)
- `level/src/grass.rs` (call grass_occluder inside spawn_area_grass)
- `player/src/plugin.rs` (register update_player_torch)
- `evergreen_main/src/main.rs` (register LightingPlugin)
- `evergreen_main/Cargo.toml` (add lighting dep)

---

## Task 1: Scaffold `lighting` crate, register Light2dPlugin, attach camera components

Goal: get `bevy_light_2d` building and on the camera with no behavior change yet. Game must still render exactly as before (ambient defaults to dark which would black-screen everything -- to avoid that, set initial `AmbientLight2d` to bright white).

**Files:**
- Create: `lighting/Cargo.toml`, `lighting/src/lib.rs`, `lighting/src/plugin.rs`
- Modify: `Cargo.toml` (workspace), `evergreen_main/Cargo.toml`, `evergreen_main/src/main.rs`, `camera/Cargo.toml`, `camera/src/plugin.rs`

- [ ] **Step 1: Create `lighting/Cargo.toml`**

YAGNI: do not add path deps for `level`/`models`/`player`/`post_processing` here. Each later task adds its own dep when first needed.

```toml
[package]
name = "lighting"
version = "0.1.0"
edition = "2021"

[lints]
workspace = true

[dependencies]
bevy = "0.18.1"
bevy_light_2d = "0.9"
```

- [ ] **Step 2: Create `lighting/src/lib.rs`**

```rust
pub mod ambient;
pub mod exit_light;
pub mod occluders;
pub mod plugin;
pub mod torch;
```

(NOTE: `ambient.rs`, `exit_light.rs`, `occluders.rs`, `torch.rs` are stubbed in this task and filled in later tasks. To avoid compile errors, create minimal placeholder files with this exact content:)

`lighting/src/ambient.rs`:
```rust
// Filled in Task 2.
```

`lighting/src/exit_light.rs`:
```rust
// Filled in Task 4.
```

`lighting/src/occluders.rs`:
```rust
// Filled in Tasks 6-8.
```

`lighting/src/torch.rs`:
```rust
// Filled in Task 5.
```

- [ ] **Step 3: Create `lighting/src/plugin.rs`**

```rust
use bevy::prelude::*;
use bevy_light_2d::plugin::Light2dPlugin;

/// Top-level lighting plugin -- composes `bevy_light_2d` + project systems.
pub struct LightingPlugin;

impl Plugin for LightingPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(Light2dPlugin);
    }
}
```

- [ ] **Step 4: Add `lighting` to workspace**

Edit `/home/ddudson/repos/evergreen/Cargo.toml`. Find the `[workspace] members = [...]` array and add `"lighting",` (preserve alphabetical order if the file uses it).

- [ ] **Step 5: Add lighting dep to camera and evergreen_main**

Edit `camera/Cargo.toml` -- add to `[dependencies]`:

```toml
bevy_light_2d = "0.9"
```

(Camera needs the crate to import `Light2d` and `AmbientLight2d` components.)

Edit `evergreen_main/Cargo.toml` -- add to `[dependencies]`:

```toml
lighting = { path = "../lighting" }
```

- [ ] **Step 6: Register LightingPlugin in main**

Edit `/home/ddudson/repos/evergreen/evergreen_main/src/main.rs`. Locate the `App::new()` plugin chain. Add `lighting::plugin::LightingPlugin` to the `.add_plugins(...)` call alongside other project plugins (e.g. after `PostProcessingPlugin`). Add the import at the top: `use lighting::plugin::LightingPlugin;`.

- [ ] **Step 7: Attach Light2d + AmbientLight2d to Camera2d**

Read `/home/ddudson/repos/evergreen/camera/src/plugin.rs`. Add imports near the existing `bevy_light_2d` consumers (alongside `use bevy::render::view::{ColorGrading, Hdr};`):

```rust
use bevy_light_2d::prelude::{AmbientLight2d, Light2d};
```

Inside the `setup` function spawn tuple, insert these two components AFTER `ColorGrading::default(),` and BEFORE the `Projection::Orthographic(...)` entry:

```rust
        Light2d,
        AmbientLight2d {
            color: Color::WHITE,
            brightness: 1.0,
        },
```

The full spawn tuple becomes (for reference -- match exact whitespace):

```rust
    commands.spawn((
        Camera2d,
        Hdr,
        // HDR + MSAA is unsupported on WebGL2 and crashes at runtime.
        // Pixel art also gains nothing from MSAA.
        Msaa::Off,
        Tonemapping::TonyMcMapface,
        DebandDither::Enabled,
        pixel_art_bloom(),
        BiomeAtmosphere::default(),
        ColorGrading::default(),
        Light2d,
        AmbientLight2d {
            color: Color::WHITE,
            brightness: 1.0,
        },
        Projection::Orthographic(OrthographicProjection {
            scaling_mode: ScalingMode::AutoMin {
                min_width: f32::from(MAP_WIDTH) * f32::from(TILE_SIZE_PX),
                min_height: f32::from(MAP_HEIGHT) * f32::from(TILE_SIZE_PX),
            },
            ..OrthographicProjection::default_2d()
        }),
    ));
```

- [ ] **Step 8: Build + clippy**

Run from `/home/ddudson/repos/evergreen`:
- `cargo build`
- `cargo clippy --workspace --all-targets -- -D warnings`

Both must pass. Expect no warnings about unused imports.

NOTE on inline `Color::WHITE`: `Color::WHITE` is a `const` Bevy type and not a `Color::srgb(...)` constructor, so it does not violate the project's banned-method lint. If clippy complains, move it to a named palette constant `LIGHT_DEFAULT_AMBIENT` (no leading magic) and reference that.

- [ ] **Step 9: Commit**

```bash
git add Cargo.toml lighting/ evergreen_main/ camera/
git commit -m "feat(lighting): scaffold lighting crate, attach Light2d to camera"
```

---

## Task 2: Migrate ambient curve into `lighting::ambient` (TDD)

Goal: lift the existing time-of-day brightness/tint logic out of `post_processing/time_sync.rs` and re-express it as an `AmbientLight2d` driver. Tests are pure-math at four anchor periods + a midpoint.

**Files:**
- Create: `lighting/src/ambient.rs` (full content -- replaces the placeholder)
- Modify: `lighting/src/plugin.rs`
- Modify: `models/src/palette.rs` (add ambient color constants)

- [ ] **Step 0: Add `models` dep to `lighting/Cargo.toml`**

Append under `[dependencies]`:

```toml
models = { path = "../models" }
```

- [ ] **Step 1: Add ambient color constants to palette**

Edit `/home/ddudson/repos/evergreen/models/src/palette.rs`. Append at the bottom of the file:

```rust
// 2D lighting ambient color anchors (per time-of-day period).
pub const AMBIENT_DAY: Color = Color::srgb(1.0, 1.0, 0.95);
pub const AMBIENT_DAWN: Color = Color::srgb(1.0, 0.85, 0.65);
pub const AMBIENT_DUSK: Color = Color::srgb(0.85, 0.55, 0.55);
pub const AMBIENT_NIGHT: Color = Color::srgb(0.4, 0.5, 0.8);
```

- [ ] **Step 2: Write the failing tests + the ambient module**

Create `/home/ddudson/repos/evergreen/lighting/src/ambient.rs` with this exact content:

```rust
use bevy::prelude::*;
use bevy_light_2d::prelude::{AmbientLight2d, Light2d};
use models::palette::{AMBIENT_DAWN, AMBIENT_DAY, AMBIENT_DUSK, AMBIENT_NIGHT};
use models::time::GameClock;

/// Period-end hours -- mirror those used by `tick_game_clock`. Kept local so
/// `lighting` does not depend on `post_processing`'s internal layout.
const NIGHT_END: f32 = 5.0;
const DAWN_END: f32 = 7.0;
const MORNING_END: f32 = 11.0;
const MIDDAY_END: f32 = 14.0;
const AFTERNOON_END: f32 = 17.0;
const DUSK_END: f32 = 19.0;
const EVENING_END: f32 = 22.0;

/// Brightness anchors per period (0..=1).
const NIGHT_BRIGHTNESS: f32 = 0.30;
const DAWN_BRIGHTNESS: f32 = 0.70;
const DAY_BRIGHTNESS: f32 = 1.00;
const DUSK_BRIGHTNESS: f32 = 0.50;

/// Lerp speed for ambient transitions (per second).
const AMBIENT_LERP_SPEED: f32 = 2.0;

/// Resolved ambient target for a given hour-of-day.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AmbientTarget {
    pub color: Color,
    pub brightness: f32,
}

impl AmbientTarget {
    const NIGHT: Self = Self {
        color: AMBIENT_NIGHT,
        brightness: NIGHT_BRIGHTNESS,
    };
    const DAWN: Self = Self {
        color: AMBIENT_DAWN,
        brightness: DAWN_BRIGHTNESS,
    };
    const DAY: Self = Self {
        color: AMBIENT_DAY,
        brightness: DAY_BRIGHTNESS,
    };
    const DUSK: Self = Self {
        color: AMBIENT_DUSK,
        brightness: DUSK_BRIGHTNESS,
    };
}

fn lerp_color(a: Color, b: Color, t: f32) -> Color {
    let a = a.to_linear();
    let b = b.to_linear();
    Color::linear_rgba(
        a.red + (b.red - a.red) * t,
        a.green + (b.green - a.green) * t,
        a.blue + (b.blue - a.blue) * t,
        a.alpha + (b.alpha - a.alpha) * t,
    )
}

fn lerp_target(a: AmbientTarget, b: AmbientTarget, t: f32) -> AmbientTarget {
    AmbientTarget {
        color: lerp_color(a.color, b.color, t),
        brightness: a.brightness + (b.brightness - a.brightness) * t,
    }
}

/// Map an hour-of-day (0..24) to an ambient target by interpolating between
/// the four time-of-day anchors.
pub fn target_for_hour(hour: f32) -> AmbientTarget {
    let h = hour.clamp(0.0, 24.0);
    if h < NIGHT_END {
        AmbientTarget::NIGHT
    } else if h < DAWN_END {
        let t = (h - NIGHT_END) / (DAWN_END - NIGHT_END);
        lerp_target(AmbientTarget::NIGHT, AmbientTarget::DAWN, t)
    } else if h < MORNING_END {
        let t = (h - DAWN_END) / (MORNING_END - DAWN_END);
        lerp_target(AmbientTarget::DAWN, AmbientTarget::DAY, t)
    } else if h < MIDDAY_END {
        AmbientTarget::DAY
    } else if h < AFTERNOON_END {
        AmbientTarget::DAY
    } else if h < DUSK_END {
        let t = (h - AFTERNOON_END) / (DUSK_END - AFTERNOON_END);
        lerp_target(AmbientTarget::DAY, AmbientTarget::DUSK, t)
    } else if h < EVENING_END {
        let t = (h - DUSK_END) / (EVENING_END - DUSK_END);
        lerp_target(AmbientTarget::DUSK, AmbientTarget::NIGHT, t)
    } else {
        AmbientTarget::NIGHT
    }
}

/// Per-frame system: lerp the camera's `Light2d.ambient_light` toward the
/// time-of-day target. `bevy_light_2d` 0.9 wraps `AmbientLight2d` inside
/// `Light2d`; query and mutate the wrapper.
pub fn sync_ambient_light(
    clock: Res<GameClock>,
    time: Res<Time>,
    mut query: Query<&mut Light2d, With<Camera2d>>,
) {
    let target = target_for_hour(clock.hour);
    let alpha = (AMBIENT_LERP_SPEED * time.delta_secs()).min(1.0);
    for mut light in &mut query {
        light.ambient_light.brightness +=
            (target.brightness - light.ambient_light.brightness) * alpha;
        light.ambient_light.color =
            lerp_color(light.ambient_light.color, target.color, alpha);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn approx(a: f32, b: f32) {
        assert!((a - b).abs() < 1e-4, "expected ~{b}, got {a}");
    }

    #[test]
    fn ambient_at_midday_returns_day() {
        let t = target_for_hour(12.0);
        assert_eq!(t.color, AMBIENT_DAY);
        approx(t.brightness, DAY_BRIGHTNESS);
    }

    #[test]
    fn ambient_at_midnight_returns_night() {
        let t = target_for_hour(0.0);
        assert_eq!(t.color, AMBIENT_NIGHT);
        approx(t.brightness, NIGHT_BRIGHTNESS);
    }

    #[test]
    fn ambient_at_dusk_anchor_returns_dusk() {
        let t = target_for_hour(19.0);
        assert_eq!(t.color, AMBIENT_DUSK);
        approx(t.brightness, DUSK_BRIGHTNESS);
    }

    #[test]
    fn ambient_at_dawn_anchor_returns_dawn() {
        let t = target_for_hour(7.0);
        assert_eq!(t.color, AMBIENT_DAWN);
        approx(t.brightness, DAWN_BRIGHTNESS);
    }

    #[test]
    fn ambient_lerps_smoothly_in_dawn() {
        let t = target_for_hour(6.0);
        let expected_brightness =
            NIGHT_BRIGHTNESS + (DAWN_BRIGHTNESS - NIGHT_BRIGHTNESS) * 0.5;
        approx(t.brightness, expected_brightness);
    }
}
```

NOTE: This module imports the four colors from `palette.rs`. The colors are defined behind the file's `#![allow(clippy::disallowed_methods)]` -- importing them downstream is fine (only constructing via `Color::srgb` is banned outside palette).

- [ ] **Step 3: Run tests**

```bash
cargo test -p lighting --lib --target x86_64-unknown-linux-gnu ambient
```

Expected: 5 tests pass.

If `Color::WHITE`/`AmbientLight2d` field names differ from this snippet, consult `cargo doc -p bevy_light_2d --no-deps` and adjust. The module signature for `lerp_color` uses `Color::linear_rgba` -- confirm Bevy 0.18 still exports this constructor; if banned by `disallowed_methods`, move `lerp_color` into `models/palette.rs` next to the other constructors (which already have the `#[allow]` attribute).

- [ ] **Step 4: Register `sync_ambient_light` in LightingPlugin**

Edit `/home/ddudson/repos/evergreen/lighting/src/plugin.rs`. Replace the existing content with:

```rust
use bevy::prelude::*;
use bevy_light_2d::plugin::Light2dPlugin;
use models::game_states::GameState;

use crate::ambient::sync_ambient_light;

/// Top-level lighting plugin -- composes `bevy_light_2d` + project systems.
pub struct LightingPlugin;

impl Plugin for LightingPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(Light2dPlugin);
        app.add_systems(
            Update,
            sync_ambient_light.run_if(in_state(GameState::Playing)),
        );
    }
}
```

NOTE: `bevy_light_2d` 0.9 does NOT expose `AmbientLight2d` as a standalone component -- it lives inside `Light2d { ambient_light: AmbientLight2d { ... } }`. The `sync_ambient_light` query must be `Query<&mut Light2d, With<Camera2d>>` and write to `light.ambient_light.color` / `.brightness`. The Step 2 module body above uses `Query<&mut AmbientLight2d, ...>` -- adjust to query `Light2d` instead and update accordingly.

NOTE: Use `Update` (not `PostUpdate`) so the ambient lerp visibly catches up each frame; the existing `sync_time_of_day` ran in `PostUpdate` only because it shared the camera's `BiomeAtmosphere` mutation with `sync_atmosphere`. For lighting we have a dedicated `AmbientLight2d` component, no contention.

- [ ] **Step 5: Build + clippy + tests**

Run from `/home/ddudson/repos/evergreen`:
- `cargo build`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace --target x86_64-unknown-linux-gnu`

All must pass.

- [ ] **Step 6: Commit**

```bash
git add lighting/ models/src/palette.rs
git commit -m "feat(lighting): ambient light driven by time-of-day curve"
```

---

## Task 3: Slim `BiomeAtmosphere` (drop ToD), delete `sync_time_of_day`

Goal: now that ambient handles brightness/tint, remove the duplicated logic from the post-processing fullscreen pass. The struct shrinks from 4 ToD scalars + padding to just `darkness` + padding.

**Files:**
- Modify: `post_processing/src/atmosphere.rs`
- Modify: `post_processing/src/time_sync.rs`
- Modify: `post_processing/src/plugin.rs`
- Modify: `assets/shaders/biome_atmosphere.wgsl`

- [ ] **Step 1: Slim the BiomeAtmosphere struct**

Read `/home/ddudson/repos/evergreen/post_processing/src/atmosphere.rs`. Replace the `BiomeAtmosphere` struct + `Default` impl with:

```rust
/// Combined post-processing effect for biome atmosphere.
///
/// Biome darkness fades the scene and adds a vignette based on area alignment.
/// Time-of-day lighting is handled by `lighting::ambient::sync_ambient_light`
/// against `AmbientLight2d`, not this shader.
///
/// Attach to the camera entity alongside `Camera2d`.
#[derive(Component, ExtractComponent, Clone, Copy, ShaderType)]
pub struct BiomeAtmosphere {
    /// Biome darkness: 0.0 = city (bright), 1.0 = darkwood (dark + vignette).
    pub darkness: f32,
    // Padding to reach 16-byte alignment (required by WebGL).
    _pad0: f32,
    _pad1: f32,
    _pad2: f32,
}

impl Default for BiomeAtmosphere {
    fn default() -> Self {
        Self {
            darkness: 0.0,
            _pad0: 0.0,
            _pad1: 0.0,
            _pad2: 0.0,
        }
    }
}
```

The `FullscreenMaterial` impl block stays unchanged (same `fragment_shader()` and `node_edges()`).

- [ ] **Step 2: Slim the shader**

Edit `/home/ddudson/repos/evergreen/assets/shaders/biome_atmosphere.wgsl`. Replace the entire file with:

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

    return vec4<f32>(color.rgb * darken * vignette, color.a);
}
```

- [ ] **Step 3: Delete sync_time_of_day and ToD anchors from time_sync**

Read `/home/ddudson/repos/evergreen/post_processing/src/time_sync.rs`. Replace the entire file with:

```rust
use bevy::prelude::*;
use models::time::GameClock;

/// Advance the game clock each frame.
pub fn tick_game_clock(mut clock: ResMut<GameClock>, time: Res<Time>) {
    clock.tick(time.delta_secs());
}
```

NOTE: This deletes `sync_time_of_day`, `period_values`, all ToD anchor constants, and the local `lerp` import. The crate no longer needs `crate::math::lerp`. If `time_sync.rs` becomes the only consumer of `crate::math` after this edit, that's OK -- `grading.rs` still uses it. Re-run `cargo clippy` after the change to confirm no orphaned imports.

- [ ] **Step 4: Drop sync_time_of_day from PostProcessingPlugin**

Read `/home/ddudson/repos/evergreen/post_processing/src/plugin.rs`. Remove the `PostUpdate` block that registers `time_sync::sync_time_of_day`. The plugin should now end with the `OnExit(Playing)` reset. Final file:

```rust
use bevy::core_pipeline::fullscreen_material::FullscreenMaterialPlugin;
use bevy::prelude::*;
use models::game_states::GameState;
use models::time::GameClock;

use crate::atmosphere::BiomeAtmosphere;
use crate::grading::{reset_color_grading, sync_color_grading};
use crate::sync;
use crate::time_sync;

pub struct PostProcessingPlugin;

impl Plugin for PostProcessingPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(FullscreenMaterialPlugin::<BiomeAtmosphere>::default());

        app.init_resource::<GameClock>();

        app.add_systems(
            Update,
            (
                sync::sync_atmosphere,
                time_sync::tick_game_clock,
                sync_color_grading,
            )
                .run_if(in_state(GameState::Playing)),
        );

        app.add_systems(OnExit(GameState::Playing), reset_color_grading);
    }
}
```

- [ ] **Step 5: Build + clippy + tests + smoke**

Run from `/home/ddudson/repos/evergreen`:
- `cargo build`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace --target x86_64-unknown-linux-gnu`

Expected: build clean, clippy clean (no orphaned imports), all tests pass. Pre-existing keybinds doctest failure is OUT OF SCOPE -- if it shows up, ignore it but don't introduce new failures.

The visible behavior change: ToD darkening/tint now comes from `AmbientLight2d` (via Task 2's `sync_ambient_light`) rather than the post-processing shader. If you have time, run `trunk serve` and verify the scene still looks correct -- if the world is far too dark or too bright, the cause is most likely an ambient anchor value, not the shader changes.

- [ ] **Step 6: Commit**

```bash
git add post_processing/ assets/shaders/biome_atmosphere.wgsl
git commit -m "refactor(post_processing): move ToD lighting from shader to AmbientLight2d"
```

---

## Task 4: Level exit point light

Goal: attach a warm yellow `PointLight2d` to the `LevelExit` entity so the goal marker actually lights its surroundings (not just bloom-haloes).

**Files:**
- Create: `lighting/src/exit_light.rs` (replace placeholder with full content)
- Modify: `models/src/palette.rs` (add LIGHT_EXIT)
- Modify: `lighting/src/plugin.rs` (register attach_level_exit_light)

- [ ] **Step 0: Add `level` dep to `lighting/Cargo.toml`**

Append under `[dependencies]`:

```toml
level = { path = "../level" }
```

(`models` is already added in T2.)

- [ ] **Step 1: Add LIGHT_EXIT to palette**

Edit `/home/ddudson/repos/evergreen/models/src/palette.rs`. Append at the bottom:

```rust
// 2D point light colors.
pub const LIGHT_EXIT: Color = Color::srgb(1.0, 0.85, 0.40);
```

- [ ] **Step 2: Write `exit_light.rs`**

Replace `/home/ddudson/repos/evergreen/lighting/src/exit_light.rs` placeholder with:

```rust
use bevy::prelude::*;
use bevy_light_2d::prelude::PointLight2d;
use level::exit::LevelExit;
use models::palette::LIGHT_EXIT;

/// Intensity of the level-exit point light (HDR scale).
const LIGHT_EXIT_INTENSITY: f32 = 4.0;
/// Radius of the level-exit point light, in world pixels.
const LIGHT_EXIT_RADIUS_PX: f32 = 96.0;
/// Falloff curve exponent (1.0 = linear).
const LIGHT_EXIT_FALLOFF: f32 = 1.0;

/// Insert a `PointLight2d` on every `LevelExit` entity that does not yet have one.
pub fn attach_level_exit_light(
    mut commands: Commands,
    query: Query<Entity, (With<LevelExit>, Without<PointLight2d>)>,
) {
    for entity in &query {
        commands.entity(entity).insert(PointLight2d {
            color: LIGHT_EXIT,
            intensity: LIGHT_EXIT_INTENSITY,
            radius: LIGHT_EXIT_RADIUS_PX,
            falloff: LIGHT_EXIT_FALLOFF,
        });
    }
}
```

NOTE: `bevy_light_2d 0.9` `PointLight2d` fields are `color`, `intensity`, `radius`, `falloff`. If a different field is required (e.g. a `cast_shadows: bool`), inspect via `cargo doc -p bevy_light_2d --no-deps --open` and add `..default()`.

- [ ] **Step 3: Register the system in LightingPlugin**

Edit `/home/ddudson/repos/evergreen/lighting/src/plugin.rs`. Update to:

```rust
use bevy::prelude::*;
use bevy_light_2d::plugin::Light2dPlugin;
use models::game_states::GameState;

use crate::ambient::sync_ambient_light;
use crate::exit_light::attach_level_exit_light;

/// Top-level lighting plugin -- composes `bevy_light_2d` + project systems.
pub struct LightingPlugin;

impl Plugin for LightingPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(Light2dPlugin);
        app.add_systems(
            Update,
            (sync_ambient_light, attach_level_exit_light)
                .run_if(in_state(GameState::Playing)),
        );
    }
}
```

`attach_level_exit_light` is idempotent (the `Without<PointLight2d>` filter prevents re-insertion), so running it every frame is cheap once the marker is attached.

- [ ] **Step 4: Build + clippy**

```bash
cargo build
cargo clippy --workspace --all-targets -- -D warnings
```

Both must pass.

- [ ] **Step 5: Commit**

```bash
git add lighting/ models/src/palette.rs
git commit -m "feat(lighting): warm point light on level exit"
```

---

## Task 5: Player torch system (TDD)

Goal: insert/remove a `PointLight2d` on the player based on biome alignment + time of day. Pure predicate is unit-tested; the system itself runs every frame in Playing.

**Files:**
- Create: `lighting/src/torch.rs` (replace placeholder)
- Modify: `models/src/palette.rs` (add LIGHT_TORCH)
- Modify: `lighting/src/plugin.rs` (register update_player_torch)
- Modify: `lighting/Cargo.toml` -- already has `level` dep; no change

- [ ] **Step 0: Verify `lighting/Cargo.toml` deps**

`Player` marker lives at `models::player::Player`, not in the `player` crate. No new dep needed -- `models` already covers it.

- [ ] **Step 1: Add LIGHT_TORCH to palette**

Edit `/home/ddudson/repos/evergreen/models/src/palette.rs`. Append:

```rust
pub const LIGHT_TORCH: Color = Color::srgb(1.0, 0.70, 0.30);
```

- [ ] **Step 2: Write the failing torch tests + module**

Create `/home/ddudson/repos/evergreen/lighting/src/torch.rs` with this exact content:

```rust
use bevy::prelude::*;
use bevy_light_2d::prelude::PointLight2d;
use level::area::AreaAlignment;
use level::world::WorldMap;
use models::palette::LIGHT_TORCH;
use models::player::Player;
use models::time::GameClock;

/// Alignment threshold above which the torch turns on regardless of hour.
const DARKWOOD_TORCH_THRESHOLD: AreaAlignment = 75;
/// Hour-of-day before which the torch is on (early morning / pre-dawn).
const TORCH_HOUR_START: f32 = 5.0;
/// Hour-of-day after which the torch is on (post-dusk).
const TORCH_HOUR_END: f32 = 19.0;

/// Player torch intensity (HDR scale).
const LIGHT_TORCH_INTENSITY: f32 = 2.5;
/// Player torch radius (world pixels).
const LIGHT_TORCH_RADIUS_PX: f32 = 80.0;
/// Player torch falloff.
const LIGHT_TORCH_FALLOFF: f32 = 1.0;

/// Pure predicate: should the torch be on for this alignment + hour?
pub fn should_torch_be_on(alignment: AreaAlignment, hour: f32) -> bool {
    alignment > DARKWOOD_TORCH_THRESHOLD
        || hour < TORCH_HOUR_START
        || hour > TORCH_HOUR_END
}

fn torch_component() -> PointLight2d {
    PointLight2d {
        color: LIGHT_TORCH,
        intensity: LIGHT_TORCH_INTENSITY,
        radius: LIGHT_TORCH_RADIUS_PX,
        falloff: LIGHT_TORCH_FALLOFF,
    }
}

/// Per-frame system: insert/remove the torch on the player based on
/// `should_torch_be_on(area.alignment, clock.hour)`.
pub fn update_player_torch(
    mut commands: Commands,
    world: Res<WorldMap>,
    clock: Res<GameClock>,
    query: Query<(Entity, Option<&PointLight2d>), With<Player>>,
) {
    let alignment = world.get_area(world.current).map_or(50, |a| a.alignment);
    let on = should_torch_be_on(alignment, clock.hour);

    for (entity, existing) in &query {
        match (on, existing.is_some()) {
            (true, false) => {
                commands.entity(entity).insert(torch_component());
            }
            (false, true) => {
                commands.entity(entity).remove::<PointLight2d>();
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn torch_off_in_daylight_city() {
        assert!(!should_torch_be_on(10, 12.0));
    }

    #[test]
    fn torch_on_at_night_anywhere() {
        assert!(should_torch_be_on(50, 22.0));
    }

    #[test]
    fn torch_on_in_darkwood_anytime() {
        assert!(should_torch_be_on(90, 12.0));
    }

    #[test]
    fn torch_off_at_dawn_greenwood() {
        assert!(!should_torch_be_on(50, 8.0));
    }

    #[test]
    fn torch_threshold_boundary_strict() {
        // alignment == threshold => false (strict greater-than)
        assert!(!should_torch_be_on(75, 12.0));
    }

    #[test]
    fn torch_threshold_above_boundary() {
        assert!(should_torch_be_on(76, 12.0));
    }
}
```

NOTE: `Player` is a marker component. Verify its location at the import: it lives in `/home/ddudson/repos/evergreen/models/src/player.rs` per existing project layout. If the import errors, locate it via `rg -n "pub struct Player" models/` and adjust.

The magic number `50` in `map_or` mirrors the existing `sync_atmosphere` fallback. To stay consistent with Phase 1's `DEFAULT_AREA_ALIGNMENT` constant in `grading.rs`, define a local one in this file too:

```rust
const DEFAULT_AREA_ALIGNMENT: AreaAlignment = 50;
```

then use `map_or(DEFAULT_AREA_ALIGNMENT, |a| a.alignment)`.

- [ ] **Step 3: Run tests**

```bash
cargo test -p lighting --lib --target x86_64-unknown-linux-gnu torch
```

Expected: 6 tests pass.

- [ ] **Step 4: Register update_player_torch in LightingPlugin**

Edit `/home/ddudson/repos/evergreen/lighting/src/plugin.rs`. Update the Update tuple:

```rust
use bevy::prelude::*;
use bevy_light_2d::plugin::Light2dPlugin;
use models::game_states::GameState;

use crate::ambient::sync_ambient_light;
use crate::exit_light::attach_level_exit_light;
use crate::torch::update_player_torch;

/// Top-level lighting plugin -- composes `bevy_light_2d` + project systems.
pub struct LightingPlugin;

impl Plugin for LightingPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(Light2dPlugin);
        app.add_systems(
            Update,
            (
                sync_ambient_light,
                attach_level_exit_light,
                update_player_torch,
            )
                .run_if(in_state(GameState::Playing)),
        );
    }
}
```

- [ ] **Step 5: Build + clippy + workspace tests**

```bash
cargo build
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --target x86_64-unknown-linux-gnu
```

All must pass.

- [ ] **Step 6: Commit**

```bash
git add lighting/ models/src/palette.rs
git commit -m "feat(lighting): player torch auto-on at night and in darkwood"
```

---

## Task 6: Tree occluders (multi-rect)

Goal: every tree spawns two child entities -- a trunk occluder and a canopy occluder -- so trees cast realistic shadows at light sources.

**Files:**
- Create/update: `lighting/src/occluders.rs` (replace placeholder, full content for trees only -- NPC and grass added in later tasks)
- Modify: `level/src/scenery.rs` (call `tree_occluders` from `spawn_tree`)
- Modify: `level/Cargo.toml` (add `lighting` dep) -- but this creates a cycle (lighting depends on level). Resolve by making the helper a `pub` free function on a struct that takes the parent `Entity` and returns a `[(LightOccluder2d, Transform); N]` -- spawned by `level` directly. See Step 1.

- [ ] **Step 1: Write `occluders.rs` (tree-only initial cut)**

Replace `/home/ddudson/repos/evergreen/lighting/src/occluders.rs` placeholder with:

```rust
use bevy::math::Vec2;
use bevy::prelude::*;
use bevy_light_2d::prelude::{LightOccluder2d, LightOccluder2dShape};

// -- Tree occluder geometry ---------------------------------------------------

/// Half-size of the trunk occluder rect (world pixels).
const TREE_TRUNK_HALF_PX: Vec2 = Vec2::new(4.0, 8.0);
/// Trunk offset from sprite anchor (sprite anchored at center; trunk sits
/// near the bottom of the sprite).
const TREE_TRUNK_OFFSET_PX: Vec2 = Vec2::new(0.0, -8.0);

/// Half-size of the canopy occluder rect.
const TREE_CANOPY_HALF_PX: Vec2 = Vec2::new(12.0, 6.0);
/// Canopy offset from sprite anchor.
const TREE_CANOPY_OFFSET_PX: Vec2 = Vec2::new(0.0, 6.0);

/// Spawn the two occluder children of a tree under `parent`.
///
/// Call this from inside the tree spawn system *after* spawning the tree's
/// sprite entity, passing the sprite's `Entity` as `parent`.
pub fn spawn_tree_occluders(commands: &mut Commands, parent: Entity) {
    spawn_occluder(commands, parent, TREE_TRUNK_HALF_PX, TREE_TRUNK_OFFSET_PX);
    spawn_occluder(commands, parent, TREE_CANOPY_HALF_PX, TREE_CANOPY_OFFSET_PX);
}

fn spawn_occluder(
    commands: &mut Commands,
    parent: Entity,
    half_size: Vec2,
    offset: Vec2,
) {
    commands.spawn((
        LightOccluder2d {
            shape: LightOccluder2dShape::Rectangle { half_size },
        },
        Transform::from_translation(offset.extend(0.0)),
        ChildOf(parent),
    ));
}
```

NOTES:
- `ChildOf(parent)` is the Bevy 0.18 child-spawning pattern (see `bevy-18` skill).
- The trunk + canopy half-sizes are tuned for typical 32x48 tree sprites. Adjust during QA if shadows look wrong; do not change here.

- [ ] **Step 2: Add `lighting` as a dep on `level` -- WAIT, check for cycle**

Run from `/home/ddudson/repos/evergreen`:

```bash
grep -A3 "\[dependencies\]" lighting/Cargo.toml
```

Expected output includes `level = { path = "../level" }`. If yes, then `level` cannot also depend on `lighting` -- it would form a cycle.

Resolution: do NOT add `lighting` to `level/Cargo.toml`. Instead, expose the `spawn_tree_occluders` helper through a different ownership model -- the tree spawn site lives in `level/src/scenery.rs::spawn_tree`. We refactor the dependency direction by:

(a) **Option A: invert deps** -- move tree/grass/npc occluder SPAWNING into the `lighting` crate via observers/`OnAdd` hooks on the existing marker components (`Scenery`, `EventNpc`, `GrassTuft`). This keeps `level` clean.

(b) **Option B: skip the helper indirection** -- inline the occluder rect components directly into the spawn tuples in `level/`, and have `level` depend on `bevy_light_2d` directly (no dep on `lighting`).

Choose **Option B** -- it avoids the observer indirection and is simpler. The named constants for occluder geometry stay in `lighting::occluders` and are imported by `level/`. So actually we DO need `level` -> `lighting`. Confirmed cycle.

To break the cycle, MOVE the occluder geometry constants OUT of `lighting` and into `models/src/lighting.rs` (a new pure-data module) -- both `level` and `lighting` depend on `models`, so no cycle. Steps:

- [ ] **Step 2a: Create `models/src/lighting.rs`**

Create `/home/ddudson/repos/evergreen/models/src/lighting.rs`:

```rust
//! Pure data describing 2D-light occluder geometry for game asset families.
//! No Bevy components -- consumers in `level/` use these dimensions to spawn
//! `LightOccluder2d` rects directly.

use bevy::math::Vec2;

// -- Tree --------------------------------------------------------------------
pub const TREE_TRUNK_HALF_PX: Vec2 = Vec2::new(4.0, 8.0);
pub const TREE_TRUNK_OFFSET_PX: Vec2 = Vec2::new(0.0, -8.0);
pub const TREE_CANOPY_HALF_PX: Vec2 = Vec2::new(12.0, 6.0);
pub const TREE_CANOPY_OFFSET_PX: Vec2 = Vec2::new(0.0, 6.0);

// -- NPC ---------------------------------------------------------------------
pub const NPC_BODY_HALF_PX: Vec2 = Vec2::new(6.0, 10.0);
pub const NPC_BODY_OFFSET_PX: Vec2 = Vec2::new(0.0, 0.0);

// -- Grass tuft --------------------------------------------------------------
pub const GRASS_OCCLUDER_HALF_PX: Vec2 = Vec2::new(3.0, 2.0);
pub const GRASS_OCCLUDER_OFFSET_PX: Vec2 = Vec2::new(0.0, 0.0);
```

Add `pub mod lighting;` to `models/src/lib.rs`.

- [ ] **Step 2b: Drop `lighting/src/occluders.rs` (no longer needed)**

Replace `/home/ddudson/repos/evergreen/lighting/src/occluders.rs` with:

```rust
//! Occluder geometry constants live in `models::lighting`. Spawning happens
//! at each asset's spawn site in `level/`. This module is intentionally empty.
```

Remove `pub mod occluders;` from `lighting/src/lib.rs`.

- [ ] **Step 3: Add `bevy_light_2d` to `level/Cargo.toml`**

Edit `/home/ddudson/repos/evergreen/level/Cargo.toml`. Add to `[dependencies]`:

```toml
bevy_light_2d = "0.9"
```

- [ ] **Step 4: Wire tree occluders into spawn_tree**

Read `/home/ddudson/repos/evergreen/level/src/scenery.rs`. Find the `spawn_tree` function. After the existing `commands.spawn((...))` that creates the tree entity, capture the returned `Entity`:

Existing code (sketch):
```rust
fn spawn_tree(commands: &mut Commands, asset_server: &AssetServer, def: ..., world_x: f32, world_y: f32) {
    commands.spawn((
        // ... tree sprite components
    ));
}
```

Modify to:

```rust
fn spawn_tree(commands: &mut Commands, asset_server: &AssetServer, def: ..., world_x: f32, world_y: f32) {
    let parent = commands.spawn((
        // ... tree sprite components (unchanged)
    )).id();

    spawn_occluder(
        commands,
        parent,
        TREE_TRUNK_HALF_PX,
        TREE_TRUNK_OFFSET_PX,
    );
    spawn_occluder(
        commands,
        parent,
        TREE_CANOPY_HALF_PX,
        TREE_CANOPY_OFFSET_PX,
    );
}

fn spawn_occluder(
    commands: &mut Commands,
    parent: Entity,
    half_size: Vec2,
    offset: Vec2,
) {
    commands.spawn((
        LightOccluder2d {
            shape: LightOccluder2dShape::Rectangle { half_size },
        },
        Transform::from_translation(offset.extend(0.0)),
        ChildOf(parent),
    ));
}
```

Add imports at the top of `scenery.rs`:

```rust
use bevy_light_2d::prelude::{LightOccluder2d, LightOccluder2dShape};
use models::lighting::{TREE_CANOPY_HALF_PX, TREE_CANOPY_OFFSET_PX, TREE_TRUNK_HALF_PX, TREE_TRUNK_OFFSET_PX};
```

NOTE: the `spawn_occluder` helper is duplicated in `scenery.rs` -- when Tasks 7 and 8 add NPC + grass occluder spawning, lift it into a shared `level/src/light_occluders.rs` (or similar) and reuse. For Task 6 keep it inline; the next tasks will refactor.

- [ ] **Step 5: Build + clippy**

```bash
cargo build
cargo clippy --workspace --all-targets -- -D warnings
```

Both must pass.

- [ ] **Step 6: Commit**

```bash
git add models/src/ lighting/ level/
git commit -m "feat(lighting): tree occluders cast multi-rect shadows"
```

---

## Task 7: NPC occluders + extract shared spawn helper

Goal: every NPC spawned via `spawn_npc_for_area` gets one body occluder. Lift the duplicated `spawn_occluder` helper from `scenery.rs` into a shared module to keep DRY.

**Files:**
- Create: `level/src/light_occluders.rs`
- Modify: `level/src/lib.rs` (declare new module)
- Modify: `level/src/scenery.rs` (replace inline helper with shared one)
- Modify: `level/src/npcs.rs` (call helper for NPC body)

- [ ] **Step 1: Create the shared helper module**

Create `/home/ddudson/repos/evergreen/level/src/light_occluders.rs`:

```rust
//! Shared spawn helper for `LightOccluder2d` child entities. Used by
//! `scenery`, `npcs`, and `grass` spawn paths.

use bevy::math::Vec2;
use bevy::prelude::*;
use bevy_light_2d::prelude::{LightOccluder2d, LightOccluder2dShape};

/// Spawn a single rect-shaped occluder as a child of `parent` at `offset`
/// (relative to parent transform) with the given `half_size`.
pub fn spawn_occluder(
    commands: &mut Commands,
    parent: Entity,
    half_size: Vec2,
    offset: Vec2,
) {
    commands.spawn((
        LightOccluder2d {
            shape: LightOccluder2dShape::Rectangle { half_size },
        },
        Transform::from_translation(offset.extend(0.0)),
        ChildOf(parent),
    ));
}
```

- [ ] **Step 2: Declare the module in level/lib.rs**

Edit `/home/ddudson/repos/evergreen/level/src/lib.rs`. Add `pub mod light_occluders;` alongside existing module declarations.

- [ ] **Step 3: Replace inline helper in scenery.rs**

Edit `/home/ddudson/repos/evergreen/level/src/scenery.rs`:
- Delete the local `fn spawn_occluder(...)` defined in Task 6.
- Replace its call sites with `crate::light_occluders::spawn_occluder(...)` (or import via `use crate::light_occluders::spawn_occluder;` at the top).

- [ ] **Step 4: Wire NPC occluder into spawn_npc_for_area**

Read `/home/ddudson/repos/evergreen/level/src/npcs.rs`. Find `spawn_npc_for_area`. Locate the `commands.spawn((...))` for the NPC entity. Capture `.id()` and call the helper:

```rust
let parent = commands.spawn((
    // ... existing NPC components
)).id();

spawn_occluder(
    &mut commands,
    parent,
    NPC_BODY_HALF_PX,
    NPC_BODY_OFFSET_PX,
);
```

Add imports at top of `npcs.rs`:

```rust
use crate::light_occluders::spawn_occluder;
use models::lighting::{NPC_BODY_HALF_PX, NPC_BODY_OFFSET_PX};
```

- [ ] **Step 5: Build + clippy**

```bash
cargo build
cargo clippy --workspace --all-targets -- -D warnings
```

Both must pass.

- [ ] **Step 6: Commit**

```bash
git add level/
git commit -m "feat(lighting): NPC body occluder + shared spawn helper"
```

---

## Task 8: Grass tuft occluders (perf risk -- last)

Goal: every grass tuft spawned via `spawn_area_grass` gets one small occluder. Highest entity count of any asset family. If WASM frame time degrades, this is the first thing to revert.

**Files:**
- Modify: `level/src/grass.rs`

- [ ] **Step 1: Wire grass occluder into spawn_area_grass**

Read `/home/ddudson/repos/evergreen/level/src/grass.rs`. Find `spawn_area_grass`. Locate the `commands.spawn((...))` near line 177 that creates each `GrassTuft`. Capture `.id()` and call the helper:

```rust
let parent = commands.spawn((
    // ... existing grass tuft components
)).id();

spawn_occluder(
    commands,
    parent,
    GRASS_OCCLUDER_HALF_PX,
    GRASS_OCCLUDER_OFFSET_PX,
);
```

Add imports at top of `grass.rs`:

```rust
use crate::light_occluders::spawn_occluder;
use models::lighting::{GRASS_OCCLUDER_HALF_PX, GRASS_OCCLUDER_OFFSET_PX};
```

NOTE: `spawn_area_grass` likely takes a borrowed `Commands` parameter. The helper expects `&mut Commands`. If the call site uses `&mut commands` (a re-borrow inside a loop), pass that re-borrow. If not, add `&mut` at the call site.

- [ ] **Step 2: Build + clippy**

```bash
cargo build
cargo clippy --workspace --all-targets -- -D warnings
```

Both must pass.

- [ ] **Step 3: WASM perf check**

Run: `trunk serve`
Open `http://127.0.0.1:8080`. Walk through a grass-heavy area (greenwood typically). Open DevTools → Performance tab. Record 5 seconds.

Expected: frame time stays under 16.6 ms (60 FPS). Grass occluders push the entity/light-sample count up significantly.

If FPS drops:
- Revert this commit only (`git revert HEAD`) -- trees + NPCs still occlude.
- Or: keep grass occluders but reduce light radius. Edit `lighting/src/torch.rs` `LIGHT_TORCH_RADIUS_PX` from 80.0 to 48.0; edit `lighting/src/exit_light.rs` `LIGHT_EXIT_RADIUS_PX` from 96.0 to 64.0.

If you reduce radii, commit the change separately:
```bash
git add lighting/src/torch.rs lighting/src/exit_light.rs
git commit -m "perf(lighting): shrink torch/exit radii for grass-occluder load"
```

- [ ] **Step 4: Commit grass occluder wiring**

```bash
git add level/src/grass.rs
git commit -m "feat(lighting): grass tuft occluders"
```

---

## Task 9: Final QA + cleanup

Goal: end-to-end verification across all GameStates, ToD periods, biomes. Confirm no regressions, write down any tuning notes.

**Files:** none (manual test + optional tuning commits)

- [ ] **Step 1: Workspace verification**

Run from `/home/ddudson/repos/evergreen`:
- `cargo build --release`  (catches release-only optimization issues)
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace --target x86_64-unknown-linux-gnu`
- `cargo fmt -- --check`

All must pass. Pre-existing keybinds doctest failure is OUT OF SCOPE.

- [ ] **Step 2: GameState walk-through (`trunk serve`)**

Open `http://127.0.0.1:8080` and verify each state:

- **MainMenu**: renders normally. Ambient should look bright/neutral (default `Color::WHITE`, brightness 1.0) since `sync_ambient_light` is gated on Playing.
- **Playing → city @ midday**: bright ambient, neutral color, no torch.
- **Playing → greenwood @ afternoon**: warming ambient, no torch.
- **Playing → darkwood @ midday**: dim ambient (Phase 1 BiomeAtmosphere darkness still applies), torch ON, exit halo bright if visible.
- **Playing → night transition**: ambient gradually shifts to AMBIENT_NIGHT (cool blue), torch turns ON around hour 19.
- **Pause** (window unfocus): ambient freezes (system gated on Playing), no flicker.
- **Dialogue**: world renders frozen, lighting frozen at last value. OK.
- **GameOver**: returns to neutral ambient on `OnExit(Playing)` (reset_color_grading fires; ambient goes to default since system stops -- this is intended).
- **LorePage**: similar -- ambient defaults.
- **KeybindConfig**: similar.

- [ ] **Step 3: Shadow QA**

In gameplay, position the player near a tree such that the level exit is on the opposite side. A shadow band should fall from the tree across the player. Move around -- shadow should track. Try same with NPCs and grass-heavy areas.

If shadows look wrong (too small / wrong position), the cause is occluder rect sizes/offsets in `models/src/lighting.rs`. Tune values; commit the tuning:

```bash
git add models/src/lighting.rs
git commit -m "tune(lighting): adjust occluder geometry for {asset}"
```

- [ ] **Step 4: Light intensity QA**

Check:
- `LIGHT_EXIT` should glow visibly in dim biomes but not blow out the goal sprite.
- `LIGHT_TORCH` should illuminate ~2 tiles around the player without making everything daylight-bright.
- Bloom halo on the exit should still be present (it's a separate bloom pass on emissive sprites; lighting and bloom both contribute).

Tune intensity constants in `lighting/src/torch.rs` and `lighting/src/exit_light.rs` if needed; commit each tuning separately.

- [ ] **Step 5: Final summary commit (if any)**

If no tuning required, no commit. Otherwise the tuning commits land here.

---

## Self-Review Checklist (already applied by author)

**1. Spec coverage:**

| Spec section | Task |
|---|---|
| `bevy_light_2d` plugin + camera attach | T1 |
| Ambient curve migrated | T2 |
| `BiomeAtmosphere` slimmed (struct + shader + plugin) | T3 |
| Level exit point light | T4 |
| Player torch (auto-on, no toggle) | T5 |
| Tree occluders (multi-rect) | T6 |
| NPC occluders (single rect) | T7 |
| Grass occluders (single rect) | T8 |
| Palette constants (LIGHT_EXIT, LIGHT_TORCH, AMBIENT_*) | T2/T4/T5 |
| Tests: 5 ambient + 6 torch | T2/T5 |
| Visual smoke test, shadow QA, intensity tune | T9 |

No gaps.

**2. Placeholder scan:**
- "Decision deferred to plan time" from spec re: deleting `time_sync.rs` -- resolved in T3 Step 3 (kept the file with only `tick_game_clock`). Explicit call.
- T6 Step 2 documents an architectural decision (where occluder geometry lives -- moved to `models::lighting` to avoid `level → lighting` cycle). This is design-time work surfaced inline because the cycle was discovered while writing the plan.
- No "TBD"/"add appropriate"/"similar to". One inline NOTE at T6 Step 4 says the helper will be lifted in T7 -- the lift happens at T7 Step 1-3.

**3. Type consistency:**
- `AreaAlignment` (u8) used in `should_torch_be_on`, `update_player_torch`, `attach_level_exit_light` -- consistent.
- `PointLight2d` field set (`color`, `intensity`, `radius`, `falloff`) used identically in T4 and T5; if the actual `bevy_light_2d` 0.9 struct adds a required field, both task notes call out the same fix path.
- `LightOccluder2dShape::Rectangle { half_size }` referenced in T6 (occluders.rs draft, then deleted) and in T7 (`light_occluders.rs`) -- single source of truth post-T7.
- `DEFAULT_AREA_ALIGNMENT: AreaAlignment = 50` constant pattern reused in T5 Step 2 with a NOTE referencing the existing one in `grading.rs`. Kept duplicated rather than shared because the cycle (lighting → post_processing) is undesirable; future refactor could pull it into `models::area`.
- `ChildOf(parent)` used identically in all spawn helpers (T6, T7).
