# Ambient Atmosphere: Grass, Creatures, Weather & Time of Day

**Date:** 2026-04-13
**Status:** Approved

Four independent subsystems that add ambient life and atmosphere to the world. Each can be implemented and tested independently.

---

## 1. Grass Tufts

### Summary

Small decorative grass sprites placed in patches on grass tiles, with wind-reactive sway animation driven by the weather system's wind strength.

### Sprites

3 sizes per biome, generated at native resolution (no `custom_size` scaling):

| Size | Pixels | Description |
|------|--------|-------------|
| Small | 8x8 | Single tuft, 2-3 blades |
| Medium | 16x8 | Wider clump |
| Large | 16x16 | Dense patch |

**Biome variants:**
- **City** (alignment 1-25): short, trimmed, yellow-green (manicured lawn)
- **Greenwood** (alignment 26-75): lush, varied heights, rich green
- **Darkwood** (alignment 76-100): sparse, gray-brown, wilted/dead

Asset paths: `sprites/scenery/grass/{city,greenwood,darkwood}/grass_{small,medium,large}.webp` (9 sprites total)

### Spawning

- 20-30 per area, placed only on grass tiles
- Edge inset of 2 tiles (same as decorations)
- Deterministic placement from area seed (salt offset to avoid overlapping decorations/trees)
- Uses `blended_alignment` for biome selection near borders
- Spawned in `ensure_area_spawned` alongside decorations
- Despawned with area lifecycle

### Wind Sway

A `WindSway` component (not `Rustleable` -- different behavior):
- Oscillates continuously based on `Res<WindStrength>` (f32, 0.0-1.0) set by weather system
- Sway amplitude = `base_amplitude * wind_strength`
- Each tuft gets a random phase offset from tile hash (no unison sway)
- Formula: `angle = sin(time * frequency + phase) * max_angle * wind_strength`
- `max_angle`: 0.1 radians, `frequency`: 3.0 Hz

### Z-Ordering

`Layer::World` with y-sort. Too small for reveal system.

### Components

- `GrassTuft` marker component
- `WindSway { phase: f32 }` -- per-entity phase offset

### New Files

- `models/src/grass.rs` -- `GrassTuft`, `WindSway` components
- `models/src/wind.rs` -- `WindStrength` resource
- `level/src/grass.rs` -- spawning logic, sway animation system

---

## 2. Tiny Creatures

### Summary

Ambient wildlife that spawns per-area, wanders, and flees from the player. Pure visual -- no collision, no interaction, no gameplay impact.

### Creature Types

8x8 sprites, 2 frames each (idle/move):

| Biome | Creatures | Count |
|-------|-----------|-------|
| City | Mice, pigeons, stray cats | 3 types |
| Greenwood | Butterflies, frogs, rabbits, songbirds | 4 types |
| Darkwood | Cockroaches, crows, bats, spiders | 4 types |

Asset paths: `sprites/creatures/{city,greenwood,darkwood}/{name}.webp` (11 sprite sheets, each 16x8 -- two 8x8 frames side by side)

### Spawning

- 4-8 per area, placed on grass tiles (or dirt for city)
- Deterministic initial placement from area seed
- Spawned/despawned with area lifecycle
- Uses `blended_alignment` for biome selection near borders

### AI State Machine

Three states per creature:

| State | Behavior | Duration |
|-------|----------|----------|
| `Idle` | Stationary, idle frame | 2-5 sec (random) |
| `Wander` | Random direction, 1-2 tiles/sec | 1-3 sec (random) |
| `Flee` | Away from player, 4 tiles/sec | Until player >5 tiles away |

Transitions:
- `Idle` -> `Wander`: timer expires
- `Wander` -> `Idle`: timer expires
- Any -> `Flee`: player within 3 tiles (48px)
- `Flee` -> `Idle`: player beyond 5 tiles (80px)

### Movement Types

- **Ground** (mice, frogs, cockroaches, rabbits, spiders, cats): move along ground, flip sprite horizontally to face direction
- **Flying** (butterflies, pigeons, songbirds, crows, bats): move freely, slight vertical bobbing (sin wave, 2px amplitude)

### No Collision

Creatures don't block the player, don't interact with scenery colliders, and don't participate in the reveal system. They pass through everything.

### Z-Ordering

`Layer::World` with y-sort. Natural depth sorting from position.

### Components

- `Creature` marker
- `CreatureAi { state: CreatureState, timer: Timer, speed: f32, movement: MovementType }`
- `CreatureState` enum: `Idle`, `Wander(Vec2 direction)`, `Flee`
- `MovementType` enum: `Ground`, `Flying`

### New Files

- `models/src/creature.rs` -- components and enums
- `level/src/creatures.rs` -- spawning, AI systems (state transitions, movement, animation)

---

## 3. Weather System

### Summary

A global weather state machine that drives particle effects and feeds wind strength to grass tufts. Hand-rolled entity-based particles -- no external crate.

### Weather States

| State | Wind Strength | Particles |
|-------|---------------|-----------|
| `Clear` | 0.0-0.2 | None |
| `Breezy` | 0.3-0.5 | Light leaves/petals |
| `Windy` | 0.6-0.8 | Dense leaves blown sideways |
| `Rain` | 0.5-0.7 | Rain streaks + ground splashes |
| `Storm` | 0.8-1.0 | Heavy rain + wind + dense streaks |

### State Transitions

- Checked every 3-5 game hours (via `GameClock`)
- Weighted random, biased by current area biome:

| State | City | Greenwood | Darkwood |
|-------|------|-----------|----------|
| Clear | 50% | 30% | 15% |
| Breezy | 25% | 30% | 20% |
| Windy | 10% | 15% | 25% |
| Rain | 10% | 15% | 25% |
| Storm | 5% | 10% | 15% |

- Wind strength lerps smoothly over 2 seconds when state changes

### Particle Implementation

Entity-based particles spawned within the camera viewport:
- `WeatherParticle { velocity: Vec2, lifetime: Timer, variant: ParticleVariant }`
- Spawn system emits N particles per frame relative to camera position (not per-area)
- Update system: `position += velocity * dt`, despawn when lifetime expires
- Render at `Layer::Weather` (z=15) -- above world entities, below NPC labels

### Particle Visuals

| Variant | Size | Behavior |
|---------|------|----------|
| Leaf | 8x8 | Drift horizontal + slight downward fall. Biome-colored (green/brown/paper) |
| Raindrop | 2x8 | White streak falling at ~70deg angle |
| Splash | 4x4 | Spawns on raindrop ground contact, fades alpha over 0.2s |

Leaf sprites per biome (3 total): `sprites/particles/{green_leaf,brown_leaf,paper_scrap}.webp`
Rain sprites (shared): `sprites/particles/{raindrop,splash}.webp`

### Resources

- `WindStrength(f32)` -- 0.0-1.0, read by grass sway system
- `WeatherState` resource -- current state + transition timer

### Layer Addition

Add `Weather = 15` to `Layer` enum.

### New Files

- `models/src/weather.rs` -- `WeatherState` resource, `WeatherParticle` component, `ParticleVariant` enum
- `level/src/weather.rs` -- state machine, particle spawning, particle update, wind sync
- Weather particle sprites (5 total)

---

## 4. Time of Day

### Summary

A game clock resource that drives a post-processing shader for lighting and color grading. Extends the existing `BiomeAtmosphere` FullscreenMaterial pattern.

### GameClock Resource

- `hour: f32` (0.0-24.0, wrapping)
- Rate: 1 game hour per 25 real seconds. Full day/night cycle = 10 minutes.
- Only ticks in `GameState::Playing`
- Starts at 8:00 (morning)
- Other systems query it freely (weather, future creature behavior)

### Time Periods

| Period | Hours | Brightness | Tint (R, G, B) |
|--------|-------|------------|-----------------|
| Night | 22:00-5:00 | 0.3 | (0.6, 0.7, 1.0) -- blue |
| Dawn | 5:00-7:00 | 0.3->1.0 | (0.6,0.7,1.0)->(1.0,0.9,0.8) -- orange-pink |
| Morning | 7:00-11:00 | 1.0 | (1.0, 0.98, 0.95) -- neutral warm |
| Midday | 11:00-14:00 | 1.0 | (1.0, 1.0, 0.95) -- slight warm yellow |
| Afternoon | 14:00-17:00 | 0.95 | (1.0, 0.95, 0.85) -- golden |
| Dusk | 17:00-19:00 | 0.95->0.3 | (1.0,0.95,0.85)->(0.8,0.5,0.6) -- deep orange-red |
| Evening | 19:00-22:00 | 0.3 | (0.7, 0.6, 0.9) -- purple-blue |

Values interpolate smoothly between period keyframes.

### Shader

A second fullscreen post-processing pass, chained after `BiomeAtmosphere`:
- `TimeOfDayMaterial` component on camera with: `brightness: f32`, `tint_r: f32`, `tint_g: f32`, `tint_b: f32`
- WGSL shader multiplies screen color by `vec3(tint_r, tint_g, tint_b) * brightness`
- Same `FullscreenMaterialPlugin` pattern as atmosphere

### Sync System

Runs in `PostUpdate`, reads `GameClock.hour`, interpolates brightness and tint between keyframes. Same pattern as `sync_atmosphere`.

### Interaction with Biome Atmosphere

Biome shader darkens darkwood areas. Time-of-day shader applies on top. Darkwood at night = double-dark (intentional -- dangerous feel). City at midday = bright and warm.

### New Files

- `models/src/time.rs` -- `GameClock` resource
- `post_processing/src/time_of_day.rs` -- `TimeOfDayMaterial`, shader type, sync system
- `assets/shaders/time_of_day.wgsl` -- the WGSL shader
- Register in `post_processing/src/plugin.rs`

---

## Implementation Order

1. **Time of Day** -- establishes `GameClock` resource that weather needs
2. **Weather** -- establishes `WindStrength` resource that grass needs
3. **Grass Tufts** -- reads `WindStrength`, simple spawning
4. **Tiny Creatures** -- independent, most complex (AI), last

Each gets its own plan -> implementation cycle.

---

## Scope

### In Scope
- 9 grass sprites, 11 creature sprite sheets, 5 particle sprites (via PixelLab)
- GameClock resource with configurable tick rate
- Time-of-day post-processing shader
- Weather state machine with 5 states
- Entity-based particles (leaves, rain, splash)
- WindStrength resource driving grass sway
- Creature AI (idle/wander/flee)
- All systems respect GameState (only active during Playing)

### Out of Scope
- Creature interaction/catching
- Nocturnal/diurnal creature behavior
- Weather affecting gameplay (slippery ground, reduced visibility)
- Seasonal changes
- Sound effects for weather/creatures
- Saving/restoring GameClock or WeatherState
