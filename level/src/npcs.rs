//! NPC entity spawning -- area-aware.
//!
//! Each NPC's home area is assigned randomly at world creation via the
//! `NpcHomes` resource. Only NPCs whose home matches the current area
//! are spawned; they are despawned and re-spawned on area change.

use bevy::prelude::*;
use dialog::components::{BarkPool, Talker};
use models::layer::Layer;
use models::npc_anim::{NpcAnimFrame, NpcAnimKind, NpcAnimTimer, NpcFacing, NpcSheet};
use models::scenery::SceneryCollider;

use crate::area::{MAP_HEIGHT, MAP_WIDTH};
use crate::npc_homes::{NpcHomes, NpcKind};
use crate::npc_wander::NpcWander;
use crate::spawning::TILE_SIZE_PX;
use crate::world::{AreaChanged, WorldMap};

// ---------------------------------------------------------------------------
// Tile positions
// ---------------------------------------------------------------------------

// All NPCs spawn at the path intersection, which is guaranteed dirt.
const PATH_CENTER_X: u16 = 15;
const PATH_CENTER_Y: u16 = 8;

const NPC_Z: f32 = Layer::Npc.z_f32();
const NPC_SPRITE_SIZE_PX: f32 = 32.0;
/// AABB half-extents for NPC collision -- matches the visible character body.
const NPC_COLLIDER_HALF: Vec2 = Vec2::new(7.0, 7.0);
const BARK_RADIUS_PX: f32 = 120.0;
const BARK_COOLDOWN_SECS: f32 = 20.0;

// Sprite sheet layout: 4 rows (S/E/N/W) x 8 cols (4 idle + 4 walk).
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
    homes: Res<NpcHomes>,
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
    spawn_for_area(&mut commands, &asset_server, &mut atlas_layouts, world.current, &homes);
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
    homes: Res<NpcHomes>,
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
    spawn_for_area(&mut commands, &asset_server, &mut atlas_layouts, world.current, &homes);
}

// ---------------------------------------------------------------------------
// Private helpers
// ---------------------------------------------------------------------------

fn spawn_for_area(
    commands: &mut Commands,
    asset_server: &AssetServer,
    atlas_layouts: &mut Assets<TextureAtlasLayout>,
    area: bevy::math::IVec2,
    homes: &NpcHomes,
) {
    if homes.npc_at(area, NpcKind::Mordred) {
        spawn_mordred(commands, asset_server, atlas_layouts);
    }
    if homes.npc_at(area, NpcKind::Drizella) {
        spawn_drizella(commands, asset_server, atlas_layouts);
    }
    if homes.npc_at(area, NpcKind::Bigby) {
        spawn_bigby(commands, asset_server, atlas_layouts);
    }
    if homes.npc_at(area, NpcKind::Gothel) {
        spawn_gothel(commands, asset_server, atlas_layouts);
    }
    if homes.npc_at(area, NpcKind::Morgana) {
        spawn_morgana(commands, asset_server, atlas_layouts);
    }
    if homes.npc_at(area, NpcKind::Cadwallader) {
        spawn_cadwallader(commands, asset_server, atlas_layouts);
    }
}

/// Convert map tile coordinates to world-space pixels (tile centre).
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
    let pos = tile_world_pos(PATH_CENTER_X, PATH_CENTER_Y);
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
