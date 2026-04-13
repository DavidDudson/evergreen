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

/// Inset from area edges in tiles.
const EDGE_INSET: u16 = 3;

/// Unique salt to avoid overlapping other spawn positions.
const CREATURE_SEED_SALT: u32 = 77_777;

/// Flee speed in pixels per second (4 tiles/sec at 16px/tile).
const FLEE_SPEED_PX: f32 = 64.0;

/// Distance in pixels at which creatures start fleeing (3 tiles).
const FLEE_TRIGGER_DISTANCE_PX: f32 = 48.0;
/// Distance in pixels at which creatures stop fleeing (5 tiles).
const FLEE_STOP_DISTANCE_PX: f32 = 80.0;

/// Vertical bobbing amplitude for flying creatures (pixels).
const FLY_BOB_AMPLITUDE_PX: f32 = 2.0;
/// Vertical bobbing frequency for flying creatures (Hz).
const FLY_BOB_FREQUENCY_HZ: f32 = 3.0;

/// Sprite size for all creatures (pixels).
const CREATURE_SPRITE_SIZE_PX: f32 = 8.0;

// ---------------------------------------------------------------------------
// Speeds (pixels per second)
// ---------------------------------------------------------------------------

/// Mouse wander speed.
const SPEED_MOUSE_PX: f32 = 32.0;
/// Pigeon wander speed.
const SPEED_PIGEON_PX: f32 = 48.0;
/// Cat wander speed.
const SPEED_CAT_PX: f32 = 40.0;
/// Butterfly wander speed.
const SPEED_BUTTERFLY_PX: f32 = 24.0;
/// Frog wander speed.
const SPEED_FROG_PX: f32 = 48.0;
/// Rabbit wander speed.
const SPEED_RABBIT_PX: f32 = 56.0;
/// Songbird wander speed.
const SPEED_SONGBIRD_PX: f32 = 40.0;
/// Cockroach wander speed.
const SPEED_COCKROACH_PX: f32 = 40.0;
/// Crow wander speed.
const SPEED_CROW_PX: f32 = 48.0;
/// Bat wander speed.
const SPEED_BAT_PX: f32 = 56.0;
/// Spider wander speed.
const SPEED_SPIDER_PX: f32 = 32.0;

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
        speed: SPEED_MOUSE_PX,
    },
    CreatureDef {
        path: "sprites/creatures/city/pigeon.webp",
        movement: MovementType::Flying,
        speed: SPEED_PIGEON_PX,
    },
    CreatureDef {
        path: "sprites/creatures/city/cat.webp",
        movement: MovementType::Ground,
        speed: SPEED_CAT_PX,
    },
];

const GREENWOOD_CREATURES: &[CreatureDef] = &[
    CreatureDef {
        path: "sprites/creatures/greenwood/butterfly.webp",
        movement: MovementType::Flying,
        speed: SPEED_BUTTERFLY_PX,
    },
    CreatureDef {
        path: "sprites/creatures/greenwood/frog.webp",
        movement: MovementType::Ground,
        speed: SPEED_FROG_PX,
    },
    CreatureDef {
        path: "sprites/creatures/greenwood/rabbit.webp",
        movement: MovementType::Ground,
        speed: SPEED_RABBIT_PX,
    },
    CreatureDef {
        path: "sprites/creatures/greenwood/songbird.webp",
        movement: MovementType::Flying,
        speed: SPEED_SONGBIRD_PX,
    },
];

const DARKWOOD_CREATURES: &[CreatureDef] = &[
    CreatureDef {
        path: "sprites/creatures/darkwood/cockroach.webp",
        movement: MovementType::Ground,
        speed: SPEED_COCKROACH_PX,
    },
    CreatureDef {
        path: "sprites/creatures/darkwood/crow.webp",
        movement: MovementType::Flying,
        speed: SPEED_CROW_PX,
    },
    CreatureDef {
        path: "sprites/creatures/darkwood/bat.webp",
        movement: MovementType::Flying,
        speed: SPEED_BAT_PX,
    },
    CreatureDef {
        path: "sprites/creatures/darkwood/spider.webp",
        movement: MovementType::Ground,
        speed: SPEED_SPIDER_PX,
    },
];

// ---------------------------------------------------------------------------
// Spawning
// ---------------------------------------------------------------------------

/// Spawn 4-8 creatures for a single area.
pub fn spawn_area_creatures(
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

    let creature_seed = area_seed.wrapping_add(CREATURE_SEED_SALT);

    // Collect candidate tiles: grass for any biome, or dirt (city has lots of dirt).
    let inset = u32::from(EDGE_INSET);
    let x_max = u32::from(MAP_WIDTH).saturating_sub(inset);
    let y_max = u32::from(MAP_HEIGHT).saturating_sub(inset);
    let mut candidates: Vec<(u32, u32)> = Vec::new();
    for x in inset..x_max {
        for y in inset..y_max {
            let terrain = area.terrain_at(x, y);
            let biome = Biome::from_alignment(area.alignment);
            let valid = match biome {
                Biome::City => terrain == Some(Terrain::Grass) || terrain == Some(Terrain::Dirt),
                _ => terrain == Some(Terrain::Grass),
            };
            if valid {
                candidates.push((x, y));
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
                custom_size: Some(Vec2::splat(CREATURE_SPRITE_SIZE_PX)),
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
    let player_pos = player_q.iter().next().map(|t| t.translation.truncate());

    for (tf, mut ai) in &mut creature_q {
        let creature_pos = tf.translation.truncate();

        // Check flee trigger from any state.
        if let Some(pp) = player_pos {
            let dist = creature_pos.distance(pp);
            if dist < FLEE_TRIGGER_DISTANCE_PX && !matches!(ai.state, CreatureState::Flee) {
                ai.state = CreatureState::Flee;
                ai.timer = Timer::from_seconds(0.0, TimerMode::Once);
                continue;
            }
            // Check flee exit.
            if matches!(ai.state, CreatureState::Flee) && dist > FLEE_STOP_DISTANCE_PX {
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
    mut creature_q: Query<(&mut Transform, &CreatureAi), (With<Creature>, Without<Player>)>,
) {
    let dt = time.delta_secs();
    let player_pos = player_q.iter().next().map(|t| t.translation.truncate());

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

/// Flip ground creature sprites horizontally based on movement direction.
pub fn creature_animation(mut query: Query<(&CreatureAi, &mut Sprite), With<Creature>>) {
    for (ai, mut sprite) in &mut query {
        if ai.movement == MovementType::Ground {
            let x_vel = match &ai.state {
                CreatureState::Wander(dir) => dir.x,
                CreatureState::Flee | CreatureState::Idle => 0.0,
            };
            if x_vel < 0.0 {
                sprite.flip_x = true;
            } else if x_vel > 0.0 {
                sprite.flip_x = false;
            }
        }
    }
}

/// Apply vertical bobbing to flying creatures.
///
/// Called after creature_movement so the bob is additive on top of movement.
pub fn creature_flying_bob(
    time: Res<Time>,
    mut query: Query<(&CreatureAi, &mut Transform), With<Creature>>,
) {
    let elapsed = time.elapsed_secs();

    for (ai, mut tf) in &mut query {
        if ai.movement == MovementType::Flying && !matches!(ai.state, CreatureState::Idle) {
            let bob = (elapsed * FLY_BOB_FREQUENCY_HZ).sin() * FLY_BOB_AMPLITUDE_PX;
            tf.translation.y += bob * time.delta_secs();
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
