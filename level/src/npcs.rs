//! NPC entity spawning — area-aware.
//!
//! Each NPC belongs to a home area (`IVec2` grid coordinate).  Only NPCs
//! whose home matches the current area are spawned; they are despawned and
//! re-spawned whenever the player crosses an area boundary.
//!
//! Characters covered:
//!  - Mordred               (Mythmakers Union PC — area (1, 0))
//!  - Drizella Tremaine     (Mythmakers Union PC — area (-1, 0))
//!  - Bigby                 (Mythmakers Union PC — area (0, 1))
//!  - Dame Gothel           (former All-Mother — area (-1, 1))
//!  - Morgana Le Fay        (The Begot Queen — area (1, 1))
//!  - Memphis Cadwallader   (The Imp — area (0, 0))

use bevy::math::IVec2;
use bevy::prelude::*;
use dialog::components::{BarkPool, Talker};
use models::layer::Layer;
use models::npc_anim::{NpcAnimFrame, NpcAnimKind, NpcAnimTimer, NpcFacing, NpcSheet};
use models::scenery::SceneryCollider;

use crate::area::{MAP_HEIGHT, MAP_WIDTH};
use crate::npc_wander::NpcWander;
use crate::spawning::TILE_SIZE_PX;
use crate::world::{AreaChanged, WorldMap};

// ---------------------------------------------------------------------------
// Home areas (world-grid coordinates)
// ---------------------------------------------------------------------------

const AREA_MORDRED: IVec2 = IVec2::new(1, 0);
const AREA_DRIZELLA: IVec2 = IVec2::new(-1, 0);
const AREA_BIGBY: IVec2 = IVec2::new(0, 1);
const AREA_GOTHEL: IVec2 = IVec2::new(-1, 1);
const AREA_MORGANA: IVec2 = IVec2::new(1, 1);
const AREA_CADWALLADER: IVec2 = IVec2::new(0, -1);

// ---------------------------------------------------------------------------
// Path-tile positions (map tile coords, 0-indexed, y=0 is bottom row)
//
// The path intersection (col 15, row 8) is guaranteed dirt in every area
// because any generated area has at least one exit, and that exit's arm
// always covers the intersection rectangle (cols 14-16, rows 7-9).
// ---------------------------------------------------------------------------

const PATH_CENTER_X: u16 = 15;
const PATH_CENTER_Y: u16 = 8;

// Cadwallader stands on the N arm of area (0,-1), which has a required North
// exit (implied by the starting area's South exit). Col 15, row 12: y >= 7 ✓
const CADWALLADER_TILE_X: u16 = 15;
const CADWALLADER_TILE_Y: u16 = 12;

const NPC_Z: f32 = Layer::Npc.z_f32();
const NPC_SPRITE_SIZE_PX: f32 = 32.0;
/// AABB half-extents for NPC collision — matches the visible character body.
const NPC_COLLIDER_HALF: Vec2 = Vec2::new(7.0, 7.0);
const BARK_RADIUS_PX: f32 = 120.0;
const BARK_COOLDOWN_SECS: f32 = 20.0;

// Sprite sheet layout: 4 rows (S/E/N/W) × 8 cols (4 idle + 4 walk).
const SHEET_COLS: u32 = 8;
const SHEET_ROWS: u32 = 4;
const FRAME_SIZE_PX: u32 = 32;
const IDLE_FRAMES: usize = 4;
const WALK_FRAMES: usize = 4;

// ---------------------------------------------------------------------------
// Marker components
// ---------------------------------------------------------------------------

#[derive(Component)]
pub struct NpcMordred;

#[derive(Component)]
pub struct NpcDrizella;

#[derive(Component)]
pub struct NpcBigby;

#[derive(Component)]
pub struct NpcGothel;

#[derive(Component)]
pub struct NpcMorgana;

#[derive(Component)]
pub struct NpcCadwallader;

// ---------------------------------------------------------------------------
// Spawn / despawn systems
// ---------------------------------------------------------------------------

pub fn spawn_npcs(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    world: Res<WorldMap>,
    existing: Query<
        (),
        Or<(
            With<NpcMordred>,
            With<NpcDrizella>,
            With<NpcBigby>,
            With<NpcGothel>,
            With<NpcMorgana>,
            With<NpcCadwallader>,
        )>,
    >,
) {
    if !existing.is_empty() {
        return;
    }
    spawn_for_area(&mut commands, &asset_server, &mut atlas_layouts, world.current);
}

pub fn despawn_npcs(
    mut commands: Commands,
    q: Query<
        Entity,
        Or<(
            With<NpcMordred>,
            With<NpcDrizella>,
            With<NpcBigby>,
            With<NpcGothel>,
            With<NpcMorgana>,
            With<NpcCadwallader>,
        )>,
    >,
) {
    for entity in &q {
        commands.entity(entity).despawn();
    }
}

pub fn respawn_npcs_on_area_change(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    world: Res<WorldMap>,
    q: Query<
        Entity,
        Or<(
            With<NpcMordred>,
            With<NpcDrizella>,
            With<NpcBigby>,
            With<NpcGothel>,
            With<NpcMorgana>,
            With<NpcCadwallader>,
        )>,
    >,
    mut events: MessageReader<AreaChanged>,
) {
    if events.read().next().is_none() {
        return;
    }
    for entity in &q {
        commands.entity(entity).despawn();
    }
    spawn_for_area(&mut commands, &asset_server, &mut atlas_layouts, world.current);
}

// ---------------------------------------------------------------------------
// Private helpers
// ---------------------------------------------------------------------------

fn spawn_for_area(
    commands: &mut Commands,
    asset_server: &AssetServer,
    atlas_layouts: &mut Assets<TextureAtlasLayout>,
    area: IVec2,
) {
    if area == AREA_MORDRED {
        spawn_mordred(commands, asset_server, atlas_layouts);
    }
    if area == AREA_DRIZELLA {
        spawn_drizella(commands, asset_server, atlas_layouts);
    }
    if area == AREA_BIGBY {
        spawn_bigby(commands, asset_server, atlas_layouts);
    }
    if area == AREA_GOTHEL {
        spawn_gothel(commands, asset_server, atlas_layouts);
    }
    if area == AREA_MORGANA {
        spawn_morgana(commands, asset_server, atlas_layouts);
    }
    if area == AREA_CADWALLADER {
        spawn_cadwallader(commands, asset_server, atlas_layouts);
    }
}

/// Convert map tile coordinates to world-space pixels (tile centre).
///
/// Matches the origin convention used by the tilemap: tile (0, 0) is the
/// bottom-left corner of the map, which is offset by half the map dimensions
/// from the world origin.
fn tile_world_pos(tx: u16, ty: u16) -> Vec3 {
    let tile_px = f32::from(TILE_SIZE_PX);
    let offset_x = -(f32::from(MAP_WIDTH) * tile_px) / 2.0;
    let offset_y = -(f32::from(MAP_HEIGHT) * tile_px) / 2.0;
    Vec3::new(
        offset_x + f32::from(tx) * tile_px + tile_px / 2.0,
        offset_y + f32::from(ty) * tile_px + tile_px / 2.0,
        NPC_Z,
    )
}

fn bark_pool(asset_server: &AssetServer, paths: &[&'static str]) -> BarkPool {
    BarkPool {
        barks: paths.iter().map(|p| asset_server.load(*p)).collect(),
        trigger_radius_px: BARK_RADIUS_PX,
        cooldown: Timer::from_seconds(BARK_COOLDOWN_SECS, TimerMode::Once),
    }
}

fn npc_sheet() -> NpcSheet {
    NpcSheet {
        idle_frames: IDLE_FRAMES,
        walk_frames: WALK_FRAMES,
        cols: SHEET_COLS.try_into().expect("SHEET_COLS fits usize"),
    }
}

fn npc_sprite(
    asset_server: &AssetServer,
    atlas_layouts: &mut Assets<TextureAtlasLayout>,
    sheet_path: &'static str,
) -> Sprite {
    let layout = TextureAtlasLayout::from_grid(
        UVec2::splat(FRAME_SIZE_PX),
        SHEET_COLS,
        SHEET_ROWS,
        None,
        None,
    );
    let layout_handle = atlas_layouts.add(layout);
    Sprite {
        image: asset_server.load(sheet_path),
        texture_atlas: Some(TextureAtlas {
            layout: layout_handle,
            index: 0,
        }),
        custom_size: Some(Vec2::splat(NPC_SPRITE_SIZE_PX)),
        ..default()
    }
}

fn npc_collider() -> SceneryCollider {
    SceneryCollider {
        half_extents: NPC_COLLIDER_HALF,
        center_offset: Vec2::ZERO,
    }
}

fn npc_anim_bundle(origin: Vec2) -> (NpcFacing, NpcAnimKind, NpcSheet, NpcAnimFrame, NpcAnimTimer, NpcWander) {
    (
        NpcFacing::default(),
        NpcAnimKind::default(),
        npc_sheet(),
        NpcAnimFrame::default(),
        NpcAnimTimer::default(),
        NpcWander::new(origin),
    )
}

fn spawn_mordred(commands: &mut Commands, asset_server: &AssetServer, layouts: &mut Assets<TextureAtlasLayout>) {
    let pos = tile_world_pos(PATH_CENTER_X, PATH_CENTER_Y);
    commands.spawn((
        NpcMordred,
        Name::new("Mordred"),
        npc_sprite(asset_server, layouts, "npc_mordred_sheet.png"),
        npc_collider(),
        Transform::from_translation(pos),
        npc_anim_bundle(pos.truncate()),
        Talker::new(asset_server.load("dialogue/scripts/mordred.dialog.ron")),
        bark_pool(asset_server, &[
            "dialogue/barks/mordred_barks.dialog.ron",
            "dialogue/barks/mordred_barks2.dialog.ron",
            "dialogue/barks/mordred_barks3.dialog.ron",
        ]),
    ));
}

fn spawn_drizella(commands: &mut Commands, asset_server: &AssetServer, layouts: &mut Assets<TextureAtlasLayout>) {
    let pos = tile_world_pos(PATH_CENTER_X, PATH_CENTER_Y);
    commands.spawn((
        NpcDrizella,
        Name::new("Drizella Tremaine"),
        npc_sprite(asset_server, layouts, "npc_drizella_sheet.png"),
        npc_collider(),
        Transform::from_translation(pos),
        npc_anim_bundle(pos.truncate()),
        Talker::new(asset_server.load("dialogue/scripts/drizella.dialog.ron")),
        bark_pool(asset_server, &[
            "dialogue/barks/drizella_barks.dialog.ron",
            "dialogue/barks/drizella_barks2.dialog.ron",
            "dialogue/barks/drizella_barks3.dialog.ron",
        ]),
    ));
}

fn spawn_bigby(commands: &mut Commands, asset_server: &AssetServer, layouts: &mut Assets<TextureAtlasLayout>) {
    let pos = tile_world_pos(PATH_CENTER_X, PATH_CENTER_Y);
    commands.spawn((
        NpcBigby,
        Name::new("Bigby"),
        npc_sprite(asset_server, layouts, "npc_bigby_sheet.png"),
        npc_collider(),
        Transform::from_translation(pos),
        npc_anim_bundle(pos.truncate()),
        Talker::new(asset_server.load("dialogue/scripts/bigby.dialog.ron")),
        bark_pool(asset_server, &[
            "dialogue/barks/bigby_barks.dialog.ron",
            "dialogue/barks/bigby_barks2.dialog.ron",
            "dialogue/barks/bigby_barks3.dialog.ron",
        ]),
    ));
}

fn spawn_gothel(commands: &mut Commands, asset_server: &AssetServer, layouts: &mut Assets<TextureAtlasLayout>) {
    let pos = tile_world_pos(PATH_CENTER_X, PATH_CENTER_Y);
    commands.spawn((
        NpcGothel,
        Name::new("Dame Gothel"),
        npc_sprite(asset_server, layouts, "npc_gothel_sheet.png"),
        npc_collider(),
        Transform::from_translation(pos),
        npc_anim_bundle(pos.truncate()),
        Talker::new(asset_server.load("dialogue/scripts/mother_gothel.dialog.ron")),
        bark_pool(asset_server, &[
            "dialogue/barks/mother_gothel_barks.dialog.ron",
            "dialogue/barks/mother_gothel_barks2.dialog.ron",
            "dialogue/barks/mother_gothel_barks3.dialog.ron",
        ]),
    ));
}

fn spawn_morgana(commands: &mut Commands, asset_server: &AssetServer, layouts: &mut Assets<TextureAtlasLayout>) {
    let pos = tile_world_pos(PATH_CENTER_X, PATH_CENTER_Y);
    commands.spawn((
        NpcMorgana,
        Name::new("Morgana Le Fay"),
        npc_sprite(asset_server, layouts, "npc_morgana_sheet.png"),
        npc_collider(),
        Transform::from_translation(pos),
        npc_anim_bundle(pos.truncate()),
        Talker::new(asset_server.load("dialogue/scripts/morgana.dialog.ron")),
        bark_pool(asset_server, &[
            "dialogue/barks/morgana_barks.dialog.ron",
            "dialogue/barks/morgana_barks2.dialog.ron",
            "dialogue/barks/morgana_barks3.dialog.ron",
        ]),
    ));
}

fn spawn_cadwallader(commands: &mut Commands, asset_server: &AssetServer, layouts: &mut Assets<TextureAtlasLayout>) {
    let pos = tile_world_pos(CADWALLADER_TILE_X, CADWALLADER_TILE_Y);
    commands.spawn((
        NpcCadwallader,
        Name::new("Memphis Cadwallader"),
        npc_sprite(asset_server, layouts, "npc_cadwallader_sheet.png"),
        npc_collider(),
        Transform::from_translation(pos),
        npc_anim_bundle(pos.truncate()),
        Talker::new(asset_server.load("dialogue/scripts/cadwallader.dialog.ron")),
        bark_pool(asset_server, &[
            "dialogue/barks/cadwallader_barks.dialog.ron",
            "dialogue/barks/cadwallader_barks2.dialog.ron",
            "dialogue/barks/cadwallader_barks3.dialog.ron",
        ]),
    ));
}
