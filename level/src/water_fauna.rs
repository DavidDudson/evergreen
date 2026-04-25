//! Small creatures around water bodies: frogs sitting on plain-pond /
//! lake tiles, water striders skating on the surface, and dragonflies
//! hovering near shorelines. All are decorative -- they don't wander
//! through the full `creatures` AI; instead a cheap animation system
//! bobs them in place.

use bevy::math::{IVec2, Vec2};
use bevy::prelude::*;
use models::layer::Layer;

use crate::area::{MAP_HEIGHT, MAP_WIDTH};
use crate::spawning::{area_world_offset, TILE_SIZE_PX};
use crate::terrain::tile_hash;
use crate::world::WorldMap;

// ---------------------------------------------------------------------------
// Assets
// ---------------------------------------------------------------------------

const FROG_SPRITE: &str = "sprites/creatures/greenwood/frog.webp";
const STRIDER_SPRITE: &str = "sprites/creatures/water/water_strider.webp";
const DRAGONFLY_SPRITE: &str = "sprites/creatures/water/dragonfly.webp";
const FISH_SHADOW_SPRITE: &str = "sprites/creatures/water/fish_shadow.webp";

// ---------------------------------------------------------------------------
// Tuning
// ---------------------------------------------------------------------------

/// Per-frog-eligible-tile chance (out of 100) to spawn a frog.
const FROG_CHANCE: u32 = 35;
/// Per water-tile chance (out of 100) to spawn a water strider.
const STRIDER_CHANCE: u32 = 40;
/// Per edge-adjacent pond/lake water tile chance (out of 1000) to spawn a
/// dragonfly. Dragonflies avoid salt water and rapids -- ponds/lakes only.
const DRAGONFLY_CHANCE_PER_1000: u32 = 70;
/// Per ocean tile chance (out of 1000) to spawn a fish shadow drifter.
const FISH_CHANCE_PER_1000: u32 = 4;

/// Visual sizes (square sprites, scaled to this pixel size).
const FROG_SIZE_PX: f32 = 8.0;
const STRIDER_SIZE_PX: f32 = 6.0;
const DRAGONFLY_SIZE_PX: f32 = 10.0;
const FISH_SIZE_PX: f32 = 14.0;

/// Fish drift speed + meander params.
const FISH_SPEED_PX: f32 = 8.0;
const FISH_MEANDER_FREQ_HZ: f32 = 0.5;
const FISH_MEANDER_AMPLITUDE_PX: f32 = 4.0;

/// Dragonfly drift: Lissajous-style path so they wander rather than just bob.
const DRAGONFLY_DRIFT_RADIUS_X_PX: f32 = 14.0;
const DRAGONFLY_DRIFT_RADIUS_Y_PX: f32 = 8.0;
const DRAGONFLY_DRIFT_FREQ_X_HZ: f32 = 0.35;
const DRAGONFLY_DRIFT_FREQ_Y_HZ: f32 = 0.5;

/// Water strider horizontal skate amplitude + frequency.
const STRIDER_SKATE_AMPLITUDE_PX: f32 = 3.0;
const STRIDER_SKATE_FREQ_HZ: f32 = 0.8;

/// Frog sit-bob amplitude + frequency (subtle -- frogs are mostly still).
const FROG_BOB_AMPLITUDE_PX: f32 = 0.5;
const FROG_BOB_FREQ_HZ: f32 = 0.6;

/// Pixel dimensions of one map area.
#[allow(clippy::as_conversions)]
const MAP_W_PX: f32 = MAP_WIDTH as f32 * TILE_SIZE_PX as f32;
#[allow(clippy::as_conversions)]
const MAP_H_PX: f32 = MAP_HEIGHT as f32 * TILE_SIZE_PX as f32;

// ---------------------------------------------------------------------------
// Components
// ---------------------------------------------------------------------------

#[derive(Component)]
pub struct Frog {
    pub phase: f32,
    pub base_y: f32,
}

#[derive(Component)]
pub struct WaterStrider {
    pub phase: f32,
    pub base_x: f32,
}

#[derive(Component)]
pub struct Dragonfly {
    pub phase: f32,
    pub base_x: f32,
    pub base_y: f32,
}

#[derive(Component)]
pub struct FishShadow {
    pub phase: f32,
    pub dir_x: f32,
    pub base_y: f32,
}

/// Generic marker so teardown can despawn all water creatures in one query.
#[derive(Component)]
pub struct WaterCreature;

// ---------------------------------------------------------------------------
// Spawning
// ---------------------------------------------------------------------------

pub fn spawn_area_water_fauna(
    commands: &mut Commands,
    asset_server: &AssetServer,
    world: &WorldMap,
    area_pos: IVec2,
) {
    let base = area_world_offset(area_pos);
    let base_offset_x = base.x - MAP_W_PX / 2.0;
    let base_offset_y = base.y - MAP_H_PX / 2.0;
    let tile_px = f32::from(TILE_SIZE_PX);
    let water = &world.water;
    let ax = u32::from_ne_bytes(area_pos.x.to_ne_bytes());
    let ay = u32::from_ne_bytes(area_pos.y.to_ne_bytes());
    let area_seed = ax
        .wrapping_mul(2_654_435_761)
        .wrapping_add(ay.wrapping_mul(1_013_904_223))
        .wrapping_add(0xFA_AA);

    for (local, kind) in water.tiles_in_area(area_pos) {
        let world_x = base_offset_x
            + f32::from(u16::try_from(local.x).unwrap_or(0)) * tile_px
            + tile_px / 2.0;
        let world_y = base_offset_y
            + f32::from(u16::try_from(local.y).unwrap_or(0)) * tile_px
            + tile_px / 2.0;
        let hash = tile_hash(local.x, local.y, area_seed);

        for rule in FAUNA_RULES {
            if !(rule.applies_to)(kind) {
                continue;
            }
            if rule.edge_only && !water.is_edge_tile(area_pos, local) {
                continue;
            }
            let denom = usize::try_from(rule.chance_denominator).unwrap_or(100);
            let mult = usize::try_from(rule.hash_mult).unwrap_or(1);
            if hash.wrapping_mul(mult) % denom < usize_of(rule.chance_numerator) {
                (rule.spawn)(commands, asset_server, world_x, world_y, hash);
            }
        }
    }

    let _ = (area_seed, tile_px, base_offset_x, base_offset_y, world);
}

/// Declarative rule set for water fauna spawning. To add a new fauna type,
/// implement a `spawn_*` function and add a `FaunaRule` entry.
struct FaunaRule {
    applies_to: fn(crate::water::WaterKind) -> bool,
    edge_only: bool,
    chance_numerator: u32,
    chance_denominator: u32,
    hash_mult: u32,
    spawn: fn(&mut Commands, &AssetServer, f32, f32, usize),
}

const FAUNA_RULES: &[FaunaRule] = &[
    // Fish shadows drift under ocean tiles (very sparse -- per-mille roll).
    FaunaRule {
        applies_to: crate::water::WaterKind::is_ocean,
        edge_only: false,
        chance_numerator: FISH_CHANCE_PER_1000,
        chance_denominator: 1000,
        hash_mult: 19,
        spawn: spawn_fish,
    },
    // Water striders skate only on still water (not rivers).
    FaunaRule {
        applies_to: crate::water::WaterKind::is_still,
        edge_only: false,
        chance_numerator: STRIDER_CHANCE,
        chance_denominator: 100,
        hash_mult: 3,
        spawn: spawn_strider,
    },
    // Frogs on plain + lake tiles only, edge preferred (visually on lily pads).
    FaunaRule {
        applies_to: crate::water::WaterKind::spawns_frogs,
        edge_only: true,
        chance_numerator: FROG_CHANCE,
        chance_denominator: 100,
        hash_mult: 5,
        spawn: spawn_frog,
    },
    // Dragonflies: ponds + lakes only, shoreline, very rare.
    FaunaRule {
        applies_to: crate::water::WaterKind::spawns_lily_pads,
        edge_only: true,
        chance_numerator: DRAGONFLY_CHANCE_PER_1000,
        chance_denominator: 1000,
        hash_mult: 11,
        spawn: spawn_dragonfly,
    },
];

fn usize_of(val: u32) -> usize {
    usize::try_from(val).unwrap_or(0)
}

fn spawn_strider(commands: &mut Commands, asset_server: &AssetServer, x: f32, y: f32, hash: usize) {
    #[allow(clippy::as_conversions)]
    let phase = (hash % 628) as f32 / 100.0;
    commands.spawn((
        WaterCreature,
        WaterStrider { phase, base_x: x },
        Sprite {
            image: asset_server.load(STRIDER_SPRITE),
            custom_size: Some(Vec2::splat(STRIDER_SIZE_PX)),
            ..default()
        },
        Transform::from_xyz(x, y, Layer::Tilemap.z_f32() + 0.8),
    ));
}

fn spawn_frog(commands: &mut Commands, asset_server: &AssetServer, x: f32, y: f32, hash: usize) {
    #[allow(clippy::as_conversions)]
    let phase = (hash.wrapping_mul(31) % 628) as f32 / 100.0;
    commands.spawn((
        WaterCreature,
        Frog { phase, base_y: y },
        Sprite {
            image: asset_server.load(FROG_SPRITE),
            custom_size: Some(Vec2::splat(FROG_SIZE_PX)),
            ..default()
        },
        Transform::from_xyz(x, y, Layer::Tilemap.z_f32() + 0.9),
    ));
}

fn spawn_fish(commands: &mut Commands, asset_server: &AssetServer, x: f32, y: f32, hash: usize) {
    #[allow(clippy::as_conversions)]
    let phase = (hash.wrapping_mul(41) % 628) as f32 / 100.0;
    let dir_x = if hash.is_multiple_of(2) { 1.0 } else { -1.0 };
    commands.spawn((
        WaterCreature,
        FishShadow {
            phase,
            dir_x,
            base_y: y,
        },
        Sprite {
            image: asset_server.load(FISH_SHADOW_SPRITE),
            custom_size: Some(Vec2::splat(FISH_SIZE_PX)),
            color: models::palette::FISH_SHADOW_TINT,
            ..default()
        },
        Transform::from_xyz(x, y, Layer::Tilemap.z_f32() + 0.55),
    ));
}

fn spawn_dragonfly(
    commands: &mut Commands,
    asset_server: &AssetServer,
    x: f32,
    y: f32,
    hash: usize,
) {
    #[allow(clippy::as_conversions)]
    let phase = (hash.wrapping_mul(17) % 628) as f32 / 100.0;
    commands.spawn((
        WaterCreature,
        Dragonfly {
            phase,
            base_x: x,
            base_y: y,
        },
        Sprite {
            image: asset_server.load(DRAGONFLY_SPRITE),
            custom_size: Some(Vec2::splat(DRAGONFLY_SIZE_PX)),
            ..default()
        },
        Transform::from_xyz(x, y, Layer::Weather.z_f32() - 1.0),
    ));
}

// ---------------------------------------------------------------------------
// Animation
// ---------------------------------------------------------------------------

type FrogQuery<'w, 's> = Query<
    'w,
    's,
    (&'static Frog, &'static mut Transform),
    (
        Without<WaterStrider>,
        Without<Dragonfly>,
        Without<FishShadow>,
    ),
>;
type StriderQuery<'w, 's> = Query<
    'w,
    's,
    (&'static WaterStrider, &'static mut Transform),
    (Without<Frog>, Without<Dragonfly>, Without<FishShadow>),
>;
type DragonflyQuery<'w, 's> = Query<
    'w,
    's,
    (&'static Dragonfly, &'static mut Transform),
    (Without<Frog>, Without<WaterStrider>, Without<FishShadow>),
>;
type FishQuery<'w, 's> = Query<
    'w,
    's,
    (&'static FishShadow, &'static mut Transform),
    (Without<Frog>, Without<WaterStrider>, Without<Dragonfly>),
>;

pub fn animate_water_fauna(
    time: Res<Time>,
    mut frogs: FrogQuery,
    mut striders: StriderQuery,
    mut dragonflies: DragonflyQuery,
    mut fish: FishQuery,
) {
    let t = time.elapsed_secs();
    for (frog, mut tf) in &mut frogs {
        tf.translation.y =
            frog.base_y + (t * FROG_BOB_FREQ_HZ + frog.phase).sin() * FROG_BOB_AMPLITUDE_PX;
    }
    for (strider, mut tf) in &mut striders {
        tf.translation.x = strider.base_x
            + (t * STRIDER_SKATE_FREQ_HZ + strider.phase).sin() * STRIDER_SKATE_AMPLITUDE_PX;
    }
    for (fly, mut tf) in &mut dragonflies {
        // Lissajous wander: separate X/Y frequencies so path never repeats on
        // the same orbit -- feels more like actual dragonfly flight.
        tf.translation.x = fly.base_x
            + (t * DRAGONFLY_DRIFT_FREQ_X_HZ + fly.phase).sin() * DRAGONFLY_DRIFT_RADIUS_X_PX;
        tf.translation.y = fly.base_y
            + (t * DRAGONFLY_DRIFT_FREQ_Y_HZ + fly.phase * 1.3).cos() * DRAGONFLY_DRIFT_RADIUS_Y_PX;
    }
    let dt = time.delta_secs();
    for (fish, mut tf) in &mut fish {
        tf.translation.x += fish.dir_x * FISH_SPEED_PX * dt;
        tf.translation.y =
            fish.base_y + (t * FISH_MEANDER_FREQ_HZ + fish.phase).sin() * FISH_MEANDER_AMPLITUDE_PX;
    }
}

pub fn despawn_water_fauna(mut commands: Commands, q: Query<Entity, With<WaterCreature>>) {
    for entity in &q {
        commands.entity(entity).despawn();
    }
}
