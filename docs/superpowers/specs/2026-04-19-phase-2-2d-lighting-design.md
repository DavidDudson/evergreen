# Phase 2: 2D Lighting Design Spec

**Date:** 2026-04-19
**Status:** Approved -- ready for plan
**Depends on:** Phase 1 (HDR + Bloom + ColorGrading) -- already shipped

## Goal

Add real-time 2D lighting to the game via `bevy_light_2d` 0.9: per-light point illumination, dynamic shadow casting from occluders, and per-time-of-day ambient color. Migrate the existing time-of-day brightness/tint logic out of the `BiomeAtmosphere` fullscreen shader into the new lighting system to keep a single source of truth.

## Scope (in)

- Integrate `bevy_light_2d` 0.9 (Bevy 0.18 compatible).
- Camera-attached `Light2d` and `AmbientLight2d`.
- Three light sources:
  - **Level exit**: warm yellow `PointLight2d` (always on).
  - **Player torch**: warm amber `PointLight2d`, auto-on at night/darkwood (no toggle key).
  - **Sun/moon**: NOT a discrete light. Migrated into `AmbientLight2d.brightness + .color` driven by the existing time-of-day curve.
- Occluders on three asset families:
  - Trees: trunk rect + canopy rect (2 children).
  - NPCs: body rect (1 child).
  - Grass tufts: small rect (1 child).
- All occluders use `LightOccluder2d` with `LightOccluder2dShape::Rectangle { half_size }`. Multiple rects per asset approximate non-rectangular silhouettes.
- Slim `BiomeAtmosphere` shader: keep biome darkness + vignette; drop ToD brightness/tint fields.

## Scope (out)

- Mask-based or polygon occluders. `bevy_light_2d` 0.9 only supports rectangles. Approximated via multi-rect.
- Directional sun/moon shadows. The crate has no directional light primitive. Sun/moon expressed only as ambient color/brightness shifts.
- Toggleable player torch (auto-on only; future key-bound toggle can layer on top of `update_player_torch`).
- Lighting on UI screens (MainMenu, Pause, etc.) -- they continue to render via the same camera but should appear neutral after the existing `reset_color_grading` and the always-on default `AmbientLight2d`.

## Architecture

### Crate layout

New crate: `lighting/` (sibling to `post_processing`).

```
lighting/
├── Cargo.toml
└── src/
    ├── lib.rs
    ├── plugin.rs
    ├── ambient.rs
    ├── torch.rs
    ├── exit_light.rs
    └── occluders.rs
```

Dependencies: `bevy 0.18.1`, `bevy_light_2d 0.9`, `level`, `models`, `player`.

### Render pipeline

```
Sprites
  -> Light2d pass (multiplies by point-light contributions + ambient color)
  -> Bloom (HDR brights still halo; emissive sprites unaffected)
  -> Tonemap + ColorGrading (existing biome-driven look)
  -> BiomeAtmosphere (slim: darkness + vignette only)
  -> screen
```

The existing `Hdr`, `Tonemapping::TonyMcMapface`, `DebandDither::Enabled`, `pixel_art_bloom()`, `BiomeAtmosphere::default()`, `ColorGrading::default()`, `Msaa::Off` components on the camera all stay. Two additions: `Light2d::default()` and `AmbientLight2d::default()`.

### Components and data flow

**Camera (`Camera2d` entity, spawned in `camera::plugin::setup`):**
- `Light2d` -- marker, enables the 2D lighting render pass.
- `AmbientLight2d { color, brightness }` -- written each frame by `lighting::ambient::sync_ambient_light`.

**Light sources:**

| Entity | Component added | Color (palette) | Intensity | Radius (px) | Lifecycle |
|---|---|---|---|---|---|
| `LevelExit` | `PointLight2d` | `LIGHT_EXIT` (warm yellow) | 4.0 | 96.0 | Inserted by `attach_level_exit_light` once per spawn |
| `Player` | `PointLight2d` | `LIGHT_TORCH` (warm amber) | 2.5 | 80.0 | Inserted/removed per frame by `update_player_torch` |

**Sun/moon:** No discrete entity. `sync_ambient_light` reads `GameClock.hour`, runs the existing `period_values` lerp (migrated from `post_processing::time_sync`), and writes the result onto the camera's `AmbientLight2d`.

**Occluders (children of source sprites):**

| Asset | Children | Rect half-sizes (named constants in `occluders.rs`) |
|---|---|---|
| Tree | 2 | `TREE_TRUNK_HALF_PX`, `TREE_CANOPY_HALF_PX` |
| NPC | 1 | `NPC_BODY_HALF_PX` |
| Grass tuft | 1 | `GRASS_OCCLUDER_HALF_PX` |

Each occluder is a child entity with `LightOccluder2d`, `Transform` (offset relative to parent sprite anchor), and an inherited `GlobalTransform` from the parent.

### Player torch logic

```rust
fn should_torch_be_on(alignment: AreaAlignment, hour: f32) -> bool {
    alignment > DARKWOOD_TORCH_THRESHOLD
        || hour < TORCH_HOUR_START
        || hour > TORCH_HOUR_END
}
```

Constants:
- `DARKWOOD_TORCH_THRESHOLD: AreaAlignment = 75`
- `TORCH_HOUR_START: f32 = 5.0` (dawn end)
- `TORCH_HOUR_END: f32 = 19.0` (dusk end)

The system inserts `PointLight2d` when the predicate flips to true and removes it when false. No flicker, no fade -- abrupt on/off. (Tuning to add fade is out of scope.)

### Ambient curve (migrated from time_sync)

`AmbientLight2d.brightness` and `.color` are interpolated from four palette anchors via the existing 8-period clock (NIGHT_END, DAWN_END, MORNING_END, MIDDAY_END, AFTERNOON_END, DUSK_END, EVENING_END). Anchors:
- `AMBIENT_DAY`: `Color::srgb(1.0, 1.0, 0.95)`, brightness 1.0
- `AMBIENT_DAWN`: `Color::srgb(1.0, 0.85, 0.65)`, brightness 0.7
- `AMBIENT_DUSK`: `Color::srgb(0.85, 0.55, 0.55)`, brightness 0.5
- `AMBIENT_NIGHT`: `Color::srgb(0.4, 0.5, 0.8)`, brightness 0.3

These move from `post_processing/src/time_sync.rs` to `lighting/src/ambient.rs`. The named constants for period-end hours stay in `time_sync.rs` (still owned by the GameClock) and are imported by `ambient.rs`. (If `time_sync.rs` becomes only `tick_game_clock` + period-end constants, leave it; if it shrinks below ~30 lines and the period-end constants only feed lighting, fold them into `ambient.rs` and delete `time_sync.rs`. Decision deferred to plan time.)

### Slimming `BiomeAtmosphere`

Drop these fields (and their padding):
- `tod_brightness: f32`
- `tod_tint_r: f32`
- `tod_tint_g: f32`
- `tod_tint_b: f32`

Keep: `darkness: f32` (biome alignment driver). The struct shrinks to one f32 + padding to maintain WebGL alignment.

Drop the corresponding shader logic in `assets/shaders/biome_atmosphere.wgsl`. Final shader does only:
1. Sample input texture.
2. Compute biome darken multiplier.
3. Compute vignette.
4. Output `color.rgb * darken * vignette`.

`sync_atmosphere` in `post_processing/src/sync.rs` continues to drive `darkness`. No changes there.

## Data Flow Summary

```
GameClock.tick (every frame)
  -> sync_ambient_light reads hour, lerps anchors, writes AmbientLight2d on camera

WorldMap.current change
  -> sync_atmosphere lerps BiomeAtmosphere.darkness (existing)
  -> sync_color_grading lerps ColorGrading (existing)
  -> update_player_torch reads area.alignment, may insert/remove PointLight2d on Player

LevelExit spawn
  -> attach_level_exit_light inserts PointLight2d once

Tree/NPC/grass spawn
  -> respective spawning system invokes occluders::* helper to add LightOccluder2d children
```

## Error Handling

No new fallible code paths. All systems use queries that gracefully iterate zero rows. The torch toggle is idempotent (insert if-absent / remove if-present; the system uses `Commands::entity(...).insert(...)` and `.remove::<PointLight2d>()` which are safe).

## Testing

### Unit tests (host target via `cargo test --target x86_64-unknown-linux-gnu`)

`lighting/src/torch.rs`:
- `torch_off_in_daylight_city` -- `should_torch_be_on(10, 12.0) == false`
- `torch_on_at_night_anywhere` -- `should_torch_be_on(50, 22.0) == true`
- `torch_on_in_darkwood_anytime` -- `should_torch_be_on(90, 12.0) == true`
- `torch_off_at_dawn_greenwood` -- `should_torch_be_on(50, 8.0) == false`
- `torch_threshold_boundary` -- `should_torch_be_on(75, 12.0) == false` (strict >)
- `torch_threshold_boundary_above` -- `should_torch_be_on(76, 12.0) == true`

`lighting/src/ambient.rs`:
- `ambient_at_midday_returns_day_color` -- color and brightness exact match
- `ambient_at_midnight_returns_night_color` -- exact match
- `ambient_lerps_dawn_smoothly` -- at hour=6, brightness between night and morning anchors

### Visual smoke test (manual, `trunk serve`)

- Walk into city area at midday: bright neutral ambient, no torch, exit halo visible if exit area is city-aligned.
- Walk into darkwood area at midday: torch on, exit halo brighter against dim ambient, tree shadows behind player visible.
- Wait for dusk transition: ambient warms to dusk anchor, then cools to night, torch turns on around hour 19.
- Walk player behind a tree relative to a light source: shadow band falls across the player.
- Pause/MainMenu/Dialogue: ambient continues at current curve value (no flicker), but no torch system runs.
- Performance: hold 60 FPS in darkwood with grass occluders + torch + exit + multiple trees in view.

## Performance Notes

- Occluder count is the main risk. Trees: ~10-30 per area. NPCs: ~3-6. Grass tufts: potentially 50+ per area.
- If WASM frame time exceeds 16.6 ms, first lever is to disable grass occluders (mark a `GRASS_OCCLUDES: bool = false` constant or feature flag and short-circuit `grass_occluder` helper).
- Second lever: reduce light radius. Smaller radius = fewer occluder rects sampled per pixel.
- Third lever: drop `LIGHT_EXIT` radius from 96 to 64.

## Open Questions

None at design time. All Q1-Q7 brainstorm answers locked.

## Migration Order (informs plan, not a task list)

1. Add `bevy_light_2d 0.9` dep, create `lighting` crate scaffold, register `Light2dPlugin`. Camera gains `Light2d` + `AmbientLight2d::default()`. No behavior change yet.
2. Migrate `period_values` from `time_sync.rs` to `lighting::ambient`. Add `sync_ambient_light` system. Tests: 3 ambient tests pass.
3. Slim `BiomeAtmosphere` (struct, shader, default, sync). Verify visual: ambient curve replaces ToD, biome darkness still works.
4. Add `LIGHT_EXIT` palette constant + `PointLight2d` on `LevelExit` via `attach_level_exit_light`.
5. Add `LIGHT_TORCH` palette constant + `update_player_torch` system + 6 torch tests.
6. Add `tree_occluders` helper; wire into tree spawn site in `level/`.
7. Add `npc_occluder` helper; wire into NPC spawn site.
8. Add `grass_occluder` helper; wire into grass spawn site (high-perf risk -- last).
9. QA pass: walk all biomes, all ToD periods, verify FPS holds; tune `LIGHT_TORCH`/`LIGHT_EXIT` intensities.

## Conventions Reaffirmed

- All numeric literals as named constants (intensity, radius, threshold, hour).
- All colors in `models/src/palette.rs` (with `#[allow(clippy::disallowed_methods)]`).
- No `unwrap()`, no `as` casts.
- Files <300 lines.
- Plugin files wiring-only.
- Tests run on host target (`--target x86_64-unknown-linux-gnu`).
- Pre-commit hook auto-runs `cargo fix` and `cargo fmt`.
