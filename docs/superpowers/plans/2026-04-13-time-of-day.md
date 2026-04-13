# Time of Day Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a game clock resource and a fullscreen post-processing shader that tints the screen based on time of day, creating dawn/day/dusk/night lighting cycles.

**Architecture:** A `GameClock` resource tracks in-game hour (0-24, wrapping). A `TimeOfDayMaterial` fullscreen shader multiplies screen color by a brightness+tint value interpolated from period keyframes. The shader chains after the existing `BiomeAtmosphere` pass: `Tonemapping -> BiomeAtmosphere -> TimeOfDayMaterial -> EndMainPassPostProcessing`. The clock ticks only in `GameState::Playing` at 1 game hour per 25 real seconds (full cycle = 10 minutes).

**Tech Stack:** Rust, Bevy 0.18, WGSL, `FullscreenMaterial` trait

---

## File Structure

| Action | File | Responsibility |
|--------|------|----------------|
| Create | `models/src/time.rs` | `GameClock` resource |
| Modify | `models/src/lib.rs` | Add `pub mod time` |
| Create | `assets/shaders/time_of_day.wgsl` | Fullscreen tint+brightness shader |
| Create | `post_processing/src/time_of_day.rs` | `TimeOfDayMaterial` component, `FullscreenMaterial` impl |
| Create | `post_processing/src/time_sync.rs` | `tick_game_clock` and `sync_time_of_day` systems |
| Modify | `post_processing/src/atmosphere.rs` | Update `node_edges` to chain to `TimeOfDayMaterial` |
| Modify | `post_processing/src/lib.rs` | Add `pub mod time_of_day` and `pub mod time_sync` |
| Modify | `post_processing/src/plugin.rs` | Register `FullscreenMaterialPlugin::<TimeOfDayMaterial>`, add systems, init `GameClock` |
| Modify | `camera/src/plugin.rs` | Add `TimeOfDayMaterial::default()` to camera spawn |

---

## Task 1: Create GameClock Resource

**Files:**
- Create: `models/src/time.rs`
- Modify: `models/src/lib.rs`

- [ ] **Step 1: Create the time module**

```rust
// models/src/time.rs

use bevy::prelude::Resource;

/// Rate at which game time advances: 1 game hour per 25 real seconds.
const HOURS_PER_REAL_SECOND: f32 = 1.0 / 25.0;

/// Maximum hour value before wrapping back to 0.
const HOURS_PER_DAY: f32 = 24.0;

/// Starting hour when a new game begins (8:00 AM -- morning).
const DEFAULT_STARTING_HOUR: f32 = 8.0;

/// Tracks the in-game time of day.
///
/// `hour` ranges from 0.0 (midnight) to just under 24.0, wrapping.
/// Advances only while `GameState::Playing`.
#[derive(Resource)]
pub struct GameClock {
    /// Current hour (0.0..24.0).
    pub hour: f32,
    /// Game hours per real second.
    pub rate: f32,
}

impl Default for GameClock {
    fn default() -> Self {
        Self {
            hour: DEFAULT_STARTING_HOUR,
            rate: HOURS_PER_REAL_SECOND,
        }
    }
}

impl GameClock {
    /// Advance the clock by `delta_seconds` real time.
    pub fn tick(&mut self, delta_seconds: f32) {
        self.hour += self.rate * delta_seconds;
        if self.hour >= HOURS_PER_DAY {
            self.hour -= HOURS_PER_DAY;
        }
    }
}
```

- [ ] **Step 2: Register in models/src/lib.rs**

Add `pub mod time;` to the module list in `models/src/lib.rs` (alphabetical order, after `tile`):

```rust
pub mod time;
```

- [ ] **Step 3: Commit**

```bash
git add models/src/time.rs models/src/lib.rs
git commit -m "Add GameClock resource for time-of-day tracking"
```

---

## Task 2: Create Time-of-Day WGSL Shader

**Files:**
- Create: `assets/shaders/time_of_day.wgsl`

- [ ] **Step 1: Write the shader**

```wgsl
// assets/shaders/time_of_day.wgsl

#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput

@group(0) @binding(0) var screen_texture: texture_2d<f32>;
@group(0) @binding(1) var texture_sampler: sampler;

struct TimeOfDay {
    brightness: f32,
    tint_r: f32,
    tint_g: f32,
    tint_b: f32,
}

@group(0) @binding(2) var<uniform> settings: TimeOfDay;

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    let color = textureSample(screen_texture, texture_sampler, in.uv);
    let tint = vec3<f32>(settings.tint_r, settings.tint_g, settings.tint_b) * settings.brightness;
    return vec4<f32>(color.rgb * tint, color.a);
}
```

- [ ] **Step 2: Commit**

```bash
git add assets/shaders/time_of_day.wgsl
git commit -m "Add time-of-day fullscreen post-processing shader"
```

---

## Task 3: Create TimeOfDayMaterial Component

**Files:**
- Create: `post_processing/src/time_of_day.rs`

- [ ] **Step 1: Implement TimeOfDayMaterial with FullscreenMaterial trait**

```rust
// post_processing/src/time_of_day.rs

use bevy::core_pipeline::fullscreen_material::FullscreenMaterial;
use bevy::prelude::*;
use bevy::render::extract_component::ExtractComponent;
use bevy::render::render_graph::{InternedRenderLabel, RenderLabel};
use bevy::render::render_resource::ShaderType;
use bevy::shader::ShaderRef;

use crate::atmosphere::BiomeAtmosphere;

/// Post-processing effect that applies time-of-day lighting.
///
/// Multiplies the screen color by `tint * brightness`.
/// Attach to the camera entity alongside `Camera2d`.
#[derive(Component, ExtractComponent, Clone, Copy, ShaderType)]
pub struct TimeOfDayMaterial {
    /// Overall brightness multiplier (0.0 = black, 1.0 = full bright).
    pub brightness: f32,
    /// Red channel tint.
    pub tint_r: f32,
    /// Green channel tint.
    pub tint_g: f32,
    /// Blue channel tint.
    pub tint_b: f32,
}

impl Default for TimeOfDayMaterial {
    fn default() -> Self {
        Self {
            brightness: 1.0,
            tint_r: 1.0,
            tint_g: 0.98,
            tint_b: 0.95,
        }
    }
}

impl FullscreenMaterial for TimeOfDayMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/time_of_day.wgsl".into()
    }

    fn node_edges() -> Vec<InternedRenderLabel> {
        vec![
            BiomeAtmosphere::node_label().intern(),
            Self::node_label().intern(),
            bevy::core_pipeline::core_2d::graph::Node2d::EndMainPassPostProcessing.intern(),
        ]
    }
}
```

- [ ] **Step 2: Commit**

```bash
git add post_processing/src/time_of_day.rs
git commit -m "Add TimeOfDayMaterial component with FullscreenMaterial impl"
```

---

## Task 4: Create Clock Tick and Sync Systems

**Files:**
- Create: `post_processing/src/time_sync.rs`

- [ ] **Step 1: Implement tick_game_clock and sync_time_of_day**

```rust
// post_processing/src/time_sync.rs

use bevy::prelude::*;
use models::time::GameClock;

use crate::time_of_day::TimeOfDayMaterial;

/// Lerp speed for time-of-day transitions (per second).
const TIME_LERP_SPEED: f32 = 2.0;

// -- Period boundary hours --
const NIGHT_END: f32 = 5.0;
const DAWN_END: f32 = 7.0;
const MORNING_END: f32 = 11.0;
const MIDDAY_END: f32 = 14.0;
const AFTERNOON_END: f32 = 17.0;
const DUSK_END: f32 = 19.0;
const EVENING_END: f32 = 22.0;

// -- Night keyframe --
const NIGHT_BRIGHTNESS: f32 = 0.3;
const NIGHT_TINT_R: f32 = 0.6;
const NIGHT_TINT_G: f32 = 0.7;
const NIGHT_TINT_B: f32 = 1.0;

// -- Morning keyframe --
const MORNING_BRIGHTNESS: f32 = 1.0;
const MORNING_TINT_R: f32 = 1.0;
const MORNING_TINT_G: f32 = 0.98;
const MORNING_TINT_B: f32 = 0.95;

// -- Dawn start keyframe (same as night) --
const DAWN_START_BRIGHTNESS: f32 = NIGHT_BRIGHTNESS;
const DAWN_START_TINT_R: f32 = NIGHT_TINT_R;
const DAWN_START_TINT_G: f32 = NIGHT_TINT_G;
const DAWN_START_TINT_B: f32 = NIGHT_TINT_B;

// -- Dawn end keyframe (warm orange-pink) --
const DAWN_END_BRIGHTNESS: f32 = 1.0;
const DAWN_END_TINT_R: f32 = 1.0;
const DAWN_END_TINT_G: f32 = 0.9;
const DAWN_END_TINT_B: f32 = 0.8;

// -- Midday keyframe --
const MIDDAY_BRIGHTNESS: f32 = 1.0;
const MIDDAY_TINT_R: f32 = 1.0;
const MIDDAY_TINT_G: f32 = 1.0;
const MIDDAY_TINT_B: f32 = 0.95;

// -- Afternoon keyframe --
const AFTERNOON_BRIGHTNESS: f32 = 0.95;
const AFTERNOON_TINT_R: f32 = 1.0;
const AFTERNOON_TINT_G: f32 = 0.95;
const AFTERNOON_TINT_B: f32 = 0.85;

// -- Dusk start keyframe (same as afternoon) --
const DUSK_START_BRIGHTNESS: f32 = AFTERNOON_BRIGHTNESS;
const DUSK_START_TINT_R: f32 = AFTERNOON_TINT_R;
const DUSK_START_TINT_G: f32 = AFTERNOON_TINT_G;
const DUSK_START_TINT_B: f32 = AFTERNOON_TINT_B;

// -- Dusk end keyframe (deep orange-red) --
const DUSK_END_BRIGHTNESS: f32 = 0.3;
const DUSK_END_TINT_R: f32 = 0.8;
const DUSK_END_TINT_G: f32 = 0.5;
const DUSK_END_TINT_B: f32 = 0.6;

// -- Evening keyframe --
const EVENING_BRIGHTNESS: f32 = 0.3;
const EVENING_TINT_R: f32 = 0.7;
const EVENING_TINT_G: f32 = 0.6;
const EVENING_TINT_B: f32 = 0.9;

/// Advance the game clock each frame.
pub fn tick_game_clock(mut clock: ResMut<GameClock>, time: Res<Time>) {
    clock.tick(time.delta_secs());
}

/// Interpolate brightness and tint from the current game hour and write to
/// the camera's `TimeOfDayMaterial`.
pub fn sync_time_of_day(
    clock: Res<GameClock>,
    time: Res<Time>,
    mut query: Query<&mut TimeOfDayMaterial>,
) {
    let (target_brightness, target_r, target_g, target_b) = period_values(clock.hour);
    let alpha = (TIME_LERP_SPEED * time.delta_secs()).min(1.0);

    for mut mat in &mut query {
        mat.brightness += (target_brightness - mat.brightness) * alpha;
        mat.tint_r += (target_r - mat.tint_r) * alpha;
        mat.tint_g += (target_g - mat.tint_g) * alpha;
        mat.tint_b += (target_b - mat.tint_b) * alpha;
    }
}

/// Compute target (brightness, tint_r, tint_g, tint_b) for a given hour.
fn period_values(hour: f32) -> (f32, f32, f32, f32) {
    if hour < NIGHT_END {
        // Night (0:00 - 5:00)
        (NIGHT_BRIGHTNESS, NIGHT_TINT_R, NIGHT_TINT_G, NIGHT_TINT_B)
    } else if hour < DAWN_END {
        // Dawn (5:00 - 7:00): interpolate night -> warm orange-pink
        let t = (hour - NIGHT_END) / (DAWN_END - NIGHT_END);
        (
            lerp(DAWN_START_BRIGHTNESS, DAWN_END_BRIGHTNESS, t),
            lerp(DAWN_START_TINT_R, DAWN_END_TINT_R, t),
            lerp(DAWN_START_TINT_G, DAWN_END_TINT_G, t),
            lerp(DAWN_START_TINT_B, DAWN_END_TINT_B, t),
        )
    } else if hour < MORNING_END {
        // Morning (7:00 - 11:00)
        (
            MORNING_BRIGHTNESS,
            MORNING_TINT_R,
            MORNING_TINT_G,
            MORNING_TINT_B,
        )
    } else if hour < MIDDAY_END {
        // Midday (11:00 - 14:00)
        (
            MIDDAY_BRIGHTNESS,
            MIDDAY_TINT_R,
            MIDDAY_TINT_G,
            MIDDAY_TINT_B,
        )
    } else if hour < AFTERNOON_END {
        // Afternoon (14:00 - 17:00)
        (
            AFTERNOON_BRIGHTNESS,
            AFTERNOON_TINT_R,
            AFTERNOON_TINT_G,
            AFTERNOON_TINT_B,
        )
    } else if hour < DUSK_END {
        // Dusk (17:00 - 19:00): interpolate afternoon -> deep orange-red
        let t = (hour - AFTERNOON_END) / (DUSK_END - AFTERNOON_END);
        (
            lerp(DUSK_START_BRIGHTNESS, DUSK_END_BRIGHTNESS, t),
            lerp(DUSK_START_TINT_R, DUSK_END_TINT_R, t),
            lerp(DUSK_START_TINT_G, DUSK_END_TINT_G, t),
            lerp(DUSK_START_TINT_B, DUSK_END_TINT_B, t),
        )
    } else if hour < EVENING_END {
        // Evening (19:00 - 22:00)
        (
            EVENING_BRIGHTNESS,
            EVENING_TINT_R,
            EVENING_TINT_G,
            EVENING_TINT_B,
        )
    } else {
        // Night (22:00 - 24:00)
        (NIGHT_BRIGHTNESS, NIGHT_TINT_R, NIGHT_TINT_G, NIGHT_TINT_B)
    }
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}
```

- [ ] **Step 2: Commit**

```bash
git add post_processing/src/time_sync.rs
git commit -m "Add tick_game_clock and sync_time_of_day systems"
```

---

## Task 5: Wire Everything Into Plugins

**Files:**
- Modify: `post_processing/src/atmosphere.rs`
- Modify: `post_processing/src/lib.rs`
- Modify: `post_processing/src/plugin.rs`
- Modify: `camera/src/plugin.rs`

- [ ] **Step 1: Update BiomeAtmosphere node_edges**

In `post_processing/src/atmosphere.rs`, change `node_edges` so the chain goes to `TimeOfDayMaterial` instead of `EndMainPassPostProcessing`. This lets `TimeOfDayMaterial` define its own chain from `BiomeAtmosphere -> TimeOfDayMaterial -> EndMainPassPostProcessing`.

Replace the current `node_edges` implementation:

```rust
use bevy::core_pipeline::core_2d::graph::Node2d;
use bevy::core_pipeline::fullscreen_material::FullscreenMaterial;
use bevy::prelude::*;
use bevy::render::extract_component::ExtractComponent;
use bevy::render::render_graph::{InternedRenderLabel, RenderLabel};
use bevy::render::render_resource::ShaderType;
use bevy::shader::ShaderRef;

/// Post-processing effect that darkens the scene and adds a vignette based on
/// area alignment (0 = city/bright, 1 = darkwood/dark).
///
/// Attach to the camera entity alongside `Camera2d`.
#[derive(Component, ExtractComponent, Clone, Copy, Default, ShaderType)]
pub struct BiomeAtmosphere {
    /// 0.0 = no effect (city), 1.0 = full darkwood darkness + vignette.
    pub darkness: f32,
    // Padding to reach 16-byte alignment (required by WebGL).
    _pad0: f32,
    _pad1: f32,
    _pad2: f32,
}

impl FullscreenMaterial for BiomeAtmosphere {
    fn fragment_shader() -> ShaderRef {
        "shaders/biome_atmosphere.wgsl".into()
    }

    fn node_edges() -> Vec<InternedRenderLabel> {
        vec![
            Node2d::Tonemapping.intern(),
            Self::node_label().intern(),
            // Next link in the chain is TimeOfDayMaterial (not EndMainPassPostProcessing).
            // TimeOfDayMaterial's own node_edges completes the chain to EndMainPassPostProcessing.
            crate::time_of_day::TimeOfDayMaterial::node_label().intern(),
        ]
    }
}
```

- [ ] **Step 2: Update post_processing/src/lib.rs**

```rust
pub mod atmosphere;
pub mod plugin;
mod sync;
pub mod time_of_day;
pub mod time_sync;
```

- [ ] **Step 3: Update post_processing/src/plugin.rs**

```rust
use bevy::core_pipeline::fullscreen_material::FullscreenMaterialPlugin;
use bevy::prelude::*;
use models::game_states::GameState;
use models::time::GameClock;

use crate::atmosphere::BiomeAtmosphere;
use crate::sync;
use crate::time_of_day::TimeOfDayMaterial;
use crate::time_sync;

pub struct PostProcessingPlugin;

impl Plugin for PostProcessingPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(FullscreenMaterialPlugin::<BiomeAtmosphere>::default())
            .add_plugins(FullscreenMaterialPlugin::<TimeOfDayMaterial>::default());

        app.init_resource::<GameClock>();

        app.add_systems(
            Update,
            (
                sync::sync_atmosphere,
                time_sync::tick_game_clock,
            )
                .run_if(in_state(GameState::Playing)),
        );

        app.add_systems(
            PostUpdate,
            time_sync::sync_time_of_day.run_if(in_state(GameState::Playing)),
        );
    }
}
```

- [ ] **Step 4: Add TimeOfDayMaterial to camera spawn**

In `camera/src/plugin.rs`, add `TimeOfDayMaterial::default()` to the camera entity spawn tuple:

```rust
use bevy::camera::ScalingMode;
use bevy::prelude::*;
use level::plugin::{MAP_HEIGHT, MAP_WIDTH, TILE_SIZE_PX};
use models::game_states::GameState;

use post_processing::atmosphere::BiomeAtmosphere;
use post_processing::time_of_day::TimeOfDayMaterial;

use crate::dialogue_focus;
use crate::smooth;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<smooth::CameraOffset>();

        app.add_systems(Startup, setup);

        app.add_systems(
            Update,
            dialogue_focus::focus_on_dialogue.run_if(in_state(GameState::Dialogue)),
        );

        app.add_systems(OnExit(GameState::Dialogue), dialogue_focus::reset_camera);

        app.add_systems(
            PostUpdate,
            smooth::follow_player.run_if(in_state(GameState::Playing)),
        );
    }
}

fn setup(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        BiomeAtmosphere::default(),
        TimeOfDayMaterial::default(),
        Projection::Orthographic(OrthographicProjection {
            scaling_mode: ScalingMode::AutoMin {
                min_width: f32::from(MAP_WIDTH) * f32::from(TILE_SIZE_PX),
                min_height: f32::from(MAP_HEIGHT) * f32::from(TILE_SIZE_PX),
            },
            ..OrthographicProjection::default_2d()
        }),
    ));
}
```

- [ ] **Step 5: Verify build**

```bash
cargo build
```

Expected: compiles with no errors.

- [ ] **Step 6: Commit**

```bash
git add post_processing/src/atmosphere.rs post_processing/src/lib.rs post_processing/src/plugin.rs camera/src/plugin.rs
git commit -m "Wire time-of-day shader into post-processing pipeline and camera"
```
