# Phase 3: Polish (Weather + Shadows + Dithering) Design Spec

**Date:** 2026-04-19
**Status:** Approved -- ready for plan
**Depends on:** Phase 1 (HDR + Bloom + ColorGrading), Phase 2 (`bevy_light_2d` integration)

## Goal

Add three polish layers on top of the lit/graded pipeline: (1) atmospheric weather particles (fireflies at night/darkwood, dust motes in clear weather, fog patches in darkwood), (2) procedural drop shadows under all entities (player, NPCs, trees, grass, creatures, Galen), and (3) shader-level Bayer dither in `BiomeAtmosphere` to hide banding in dark biomes.

## Scope (in)

- Three new `ParticleVariant`s -- `Firefly`, `DustMote`, `FogPatch` -- spawned by extensions to existing `level/src/weather.rs` systems.
- Firefly pulse-flicker animation system.
- Procedural `Mesh2d` ellipse drop shadow attached as child of every player/NPC/tree/grass/creature/Galen spawn. Single shared `Mesh<Circle>` + single `ColorMaterial` (batched render).
- Per-asset shadow geometry constants in new `models::shadow` module.
- Shader-level Bayer 4x4 ordered dither in `BiomeAtmosphere` fragment, scaled by `darkness` (city = no dither, darkwood = full).
- Camera-level `DebandDither::Enabled` (Phase 1) stays.

## Scope (out)

- Per-asset baked shadow sprites (procedural mesh chosen instead -- Q4 answer C).
- Snow particles (no cold biome exists).
- Light-aware shadows that follow point-light direction (existing `bevy_light_2d` occluder shadows already cover that; drop shadows here are purely a depth cue).
- Per-asset dithering palettes (single global Bayer matrix).

## Architecture

### Crate layout

No new crate. Extensions to existing crates:

- `models/` -- new `shadow` module (pure data); extended `weather` (new variants); extended `palette` (new color constants).
- `level/` -- new `shadows` module (resource + helper); extended `weather` (new spawning + pulse animation systems).
- `player/`, `level/scenery`, `level/npcs`, `level/grass`, `level/creatures`, `level/galen` -- each spawn site calls `spawn_drop_shadow`.
- `assets/shaders/biome_atmosphere.wgsl` -- Bayer dither inserted at end of fragment.
- `assets/sprites/particles/` -- three new WebP files (`firefly.webp`, `dust_mote.webp`, `fog_patch.webp`).

### Render pipeline (unchanged from Phase 2)

```
Sprites + Mesh2d (drop shadows)
  -> Light2d pass (ambient + point lights, multiplicative)
  -> Bloom (firefly emissive >1.0 still haloes)
  -> Tonemap + ColorGrading
  -> BiomeAtmosphere (slim: darkness + vignette + Bayer dither)
  -> screen
```

Drop shadows are unlit `Mesh2d` -- they pass through `Light2dPlugin`'s pass and stay dark regardless of biome ambient. This is intended (shadows are a depth cue, not a lit surface).

## Components and Data Flow

### Weather particle additions (`models/src/weather.rs`)

Extend `ParticleVariant`:

```rust
pub enum ParticleVariant {
    GreenLeaf,
    BrownLeaf,
    PaperScrap,
    Raindrop,
    Splash,
    Firefly,    // new
    DustMote,   // new
    FogPatch,   // new
}
```

`WeatherParticle` component reused unchanged.

### Spawning systems (`level/src/weather.rs`)

Three new spawning systems, each driven by a pure predicate:

- `firefly_active(hour: f32, alignment: AreaAlignment) -> bool` -- true when `(hour > 19.0 || hour < 5.0) && alignment > 60`.
- `dust_mote_active(weather: WeatherKind) -> bool` -- true when `weather == Clear`.
- `fog_active(alignment: AreaAlignment) -> bool` -- true when `alignment > 75`.

Spawn rates (named constants):
- `FIREFLIES_PER_SEC: f32 = 0.8`
- `DUST_MOTES_PER_SEC: f32 = 1.5`
- `FOG_PATCHES_PER_SEC: f32 = 0.3`

Lifetimes:
- `FIREFLY_LIFETIME_SECS: f32 = 6.0`
- `DUST_LIFETIME_SECS: f32 = 8.0`
- `FOG_LIFETIME_SECS: f32 = 12.0`

Velocities:
- Firefly: random horizontal ±20 px/s + sine bob (frequency 1.5 Hz, amplitude 8 px).
- Dust mote: 10 px/s in wind direction + small jitter.
- Fog patch: 15 px/s in wind direction.

Spawn position: random offset within camera viewport box (existing `VIEWPORT_HALF_W_PX`/`VIEWPORT_HALF_H_PX`).

### Firefly pulse animation (`level/src/weather.rs`)

New system `animate_fireflies` queries `(WeatherParticle, &mut Sprite)` filtered to `variant == Firefly`. Each frame:

```rust
let phase = particle_phase_from_entity(entity);  // hash of entity bits
let pulse = 0.6 + 0.4 * (elapsed * FIREFLY_PULSE_FREQ_HZ + phase).sin();
sprite.color = palette::FIREFLY.with_alpha(pulse);
```

Where `FIREFLY_PULSE_FREQ_HZ: f32 = 2.5`. Other particle variants use existing `animate_weather_particles` system unchanged.

### Sprite assets

Three new WebP files in `assets/sprites/particles/`:

- `firefly.webp` -- 2x2 white pixel. Multiplied by emissive `palette::FIREFLY` (>1.0) at sprite color, so it haloes via bloom.
- `dust_mote.webp` -- 1x1 white pixel.
- `fog_patch.webp` -- 32x16 soft wispy alpha texture.

If asset creation is blocked at plan time, the 1-2 px particles can fall back to `Mesh2d(Circle::new(1.0))` with `MeshMaterial2d` -- decision deferred to plan. Fog patch genuinely needs a texture; use an existing wispy asset as placeholder if needed and revise during QA.

### Drop shadow assets resource (`level/src/shadows.rs`)

```rust
#[derive(Resource)]
pub struct DropShadowAssets {
    pub mesh: Handle<Mesh>,
    pub material: Handle<ColorMaterial>,
}

pub fn init_shadow_assets(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let mesh = meshes.add(Circle::new(1.0));
    let material = materials.add(ColorMaterial::from(palette::DROP_SHADOW));
    commands.insert_resource(DropShadowAssets { mesh, material });
}

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

const SHADOW_Z_OFFSET: f32 = -0.1;
```

### Per-asset shadow geometry (`models/src/shadow.rs`)

```rust
use bevy::math::Vec2;

pub const PLAYER_SHADOW_HALF_PX: Vec2 = Vec2::new(8.0, 3.0);
pub const PLAYER_SHADOW_OFFSET_Y_PX: f32 = -10.0;

pub const NPC_SHADOW_HALF_PX: Vec2 = Vec2::new(7.0, 2.5);
pub const NPC_SHADOW_OFFSET_Y_PX: f32 = -10.0;

pub const TREE_SHADOW_HALF_PX: Vec2 = Vec2::new(18.0, 5.0);
pub const TREE_SHADOW_OFFSET_Y_PX: f32 = 0.0; // tree anchor already at base

pub const GRASS_SHADOW_HALF_PX: Vec2 = Vec2::new(4.0, 1.5);
pub const GRASS_SHADOW_OFFSET_Y_PX: f32 = -2.0;

pub const CREATURE_SHADOW_HALF_PX: Vec2 = Vec2::new(3.0, 1.0);
pub const CREATURE_SHADOW_OFFSET_Y_PX: f32 = -3.0;

pub const GALEN_SHADOW_HALF_PX: Vec2 = Vec2::new(7.0, 2.5);
pub const GALEN_SHADOW_OFFSET_Y_PX: f32 = -10.0;
```

### Spawn-site changes (one helper call each)

Each spawn site captures `.id()` of the parent and calls `spawn_drop_shadow` with the asset family's geometry:

- `player/src/spawning.rs::spawn` -> `PLAYER_SHADOW_*`
- `level/src/scenery.rs::spawn_tree` -> `TREE_SHADOW_*`
- `level/src/npcs.rs::spawn_npc` -> `NPC_SHADOW_*`
- `level/src/grass.rs::spawn_area_grass` -> `GRASS_SHADOW_*`
- `level/src/creatures.rs` (the spawn site) -> `CREATURE_SHADOW_*`
- `level/src/galen.rs` (the spawn site) -> `GALEN_SHADOW_*`

Each system gains a `Res<DropShadowAssets>` parameter.

### Palette additions (`models/src/palette.rs`)

```rust
// Weather particle colors.
pub const FIREFLY: Color = Color::srgb(2.0, 3.0, 0.5);     // emissive >1.0 -> bloom
pub const DUST_MOTE: Color = Color::srgb(0.9, 0.85, 0.75);
pub const FOG: Color = Color::srgba(0.6, 0.65, 0.7, 0.35);

// Drop shadow.
pub const DROP_SHADOW: Color = Color::srgba(0.0, 0.0, 0.0, 0.4);
```

### Shader Bayer dither (`assets/shaders/biome_atmosphere.wgsl`)

Replace the final `return` with:

```wgsl
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

    let darken = 1.0 - settings.darkness * 0.80;

    let center = in.uv - vec2<f32>(0.5, 0.5);
    let dist = length(center);
    let vignette_radius = 0.15;
    let vignette_soft = 0.35;
    let raw_vignette = smoothstep(vignette_radius, vignette_radius + vignette_soft, dist);
    let vignette = 1.0 - raw_vignette * settings.darkness;

    var rgb = color.rgb * darken * vignette;

    let frag_coord = vec2<u32>(in.position.xy);
    let dither = (bayer_4x4(frag_coord) - 0.5) * DITHER_STEP * settings.darkness;
    rgb = rgb + vec3<f32>(dither, dither, dither);

    return vec4<f32>(rgb, color.a);
}
```

`bayer_4x4` is a fragment-coordinate-driven 4x4 ordered matrix. Output range `[-0.5/255, +0.5/255] * darkness` adds at most ±half a color step, scaled out to zero when darkness=0. No uniform layout change; no Rust changes.

## Error Handling

No new fallible code paths. All systems use queries that gracefully iterate zero rows. `DropShadowAssets` resource is initialized once at `Startup` -- spawn sites that run before it would panic on `Res<DropShadowAssets>`. To prevent that, spawn sites already run from `Update`/`OnEnter(Playing)` (after `Startup`). Verify ordering at plan time; if any spawn site runs at `Startup` or `PreStartup`, gate the whole plugin's startup chain on a sentinel state.

## Testing

### Unit tests (host target via `cargo test --target x86_64-unknown-linux-gnu`)

`level/src/weather.rs`:
- `firefly_active_at_night_in_darkwood` -- `firefly_active(22.0, 80) == true`
- `firefly_inactive_at_midday` -- `firefly_active(12.0, 80) == false`
- `firefly_inactive_in_city_at_night` -- `firefly_active(22.0, 10) == false`
- `firefly_threshold_boundary` -- `firefly_active(22.0, 60) == false` (strict `>`)
- `dust_mote_active_during_clear` -- `dust_mote_active(WeatherKind::Clear) == true`
- `dust_mote_inactive_during_rain` -- `dust_mote_active(WeatherKind::Rain) == false`
- `fog_active_in_darkwood` -- `fog_active(80) == true`
- `fog_inactive_in_greenwood` -- `fog_active(50) == false`

8 tests total.

`level/src/shadows.rs` -- no behavior tests. Manual visual QA covers integration.

### Visual smoke test (manual, `trunk serve`)

- Walk into city @ midday/clear: shadows visible under player + scenery; no fireflies/fog; rare dust motes drift across screen.
- Walk into greenwood @ dusk: dust motes still visible (clear weather); shadows visible.
- Walk into darkwood @ midday: fog patches drift; no fireflies; shadows visible.
- Walk into darkwood @ night: fireflies appear and pulse-bloom; fog persists; shadows visible; ambient blue + biome darkness banding hidden by shader dither.
- Trigger Rain weather: dust motes stop; existing rain particles spawn; shadows visible through rain.
- Pause/Dialogue: weather particles freeze (system gated on Playing) but shadows persist as static children.

## Performance

- **Drop shadows**: one extra `Mesh2d` entity per player/NPC/tree/grass/creature/Galen. Highest count is grass (~50/area). All share one mesh + one material -> rendered as a single batched draw call.
- **Particles**: existing weather system already spawns up to ~30/sec under Storm; adding ~2.6/sec average across the three new variants is negligible.
- **Shader dither**: 4 wgsl ops per fragment. Trivial.
- **No new fullscreen passes** (dither folded into existing `BiomeAtmosphere`).

If WASM perf degrades, first lever is to disable grass shadows (`spawn_drop_shadow` skipped in `spawn_area_grass`). Second lever: lower `FIREFLIES_PER_SEC` to 0.4.

## Open Questions

None at design time. Asset-creation fallback (Mesh2d for 1-2 px particles) is documented and will be decided at plan time.

## Migration Order (informs plan, not a task list)

1. Palette: add `FIREFLY`, `DUST_MOTE`, `FOG`, `DROP_SHADOW`. Add `models::shadow` module with all six geometry constants. Extend `ParticleVariant` enum. Pure data, no behavior.
2. Create or generate the three particle WebP assets (or commit decision to use Mesh2d for fireflies/dust).
3. Add `firefly_active` predicate + spawning + pulse animation + 4 unit tests.
4. Add `dust_mote_active` predicate + spawning + 2 unit tests.
5. Add `fog_active` predicate + spawning + 2 unit tests.
6. Create `level/src/shadows.rs` with resource + startup system + helper. No spawn-site wiring yet -- verify resource initializes cleanly.
7. Wire `spawn_drop_shadow` into player spawn (single asset family, easiest verification).
8. Wire into NPC + Galen spawn sites.
9. Wire into tree + creature spawn sites.
10. Wire into grass spawn site (highest count -- perf check after).
11. Add Bayer dither to shader.
12. Final QA: visual walk-through across all biomes/ToD/weather; verify shadows render below sprites; verify dither hides banding in darkwood; verify firefly bloom halo visible at night.

## Conventions Reaffirmed

- All numeric literals as named constants (`_PX`, `_SECS`, `_PER_SEC`, `_HZ` suffixes).
- All colors in `models/src/palette.rs`.
- No `unwrap()`, no `as` casts.
- Files <300 lines.
- Plugin files wiring-only.
- Tests run on host target (`--target x86_64-unknown-linux-gnu`).
- All sprite assets WebP.
- `ChildOf(parent)` for child entity spawning.
