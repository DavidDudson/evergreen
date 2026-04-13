//! NPC entity spawning -- area-aware.
//!
//! Each NPC's home area is determined by the `AreaEvent::NpcEncounter` event
//! assigned during world generation. Only NPCs whose event matches the
//! current area are spawned.

use bevy::prelude::*;
use dialog::components::{BarkPool, Talker};
use models::layer::Layer;
use models::npc_anim::{NpcAnimFrame, NpcAnimKind, NpcAnimTimer, NpcFacing, NpcSheet};
use models::scenery::SceneryCollider;

use crate::area::{AreaEvent, MAP_HEIGHT, MAP_WIDTH, NpcKind};
use crate::npc_wander::NpcWander;
use crate::spawning::TILE_SIZE_PX;

// All NPCs spawn at the path intersection, which is guaranteed dirt.
const PATH_CENTER_X: u16 = 15;
const PATH_CENTER_Y: u16 = 8;

const NPC_Z: f32 = Layer::World.z_f32();
const NPC_SPRITE_SIZE_PX: f32 = 32.0;
const NPC_COLLIDER_HALF: Vec2 = Vec2::new(7.0, 7.0);
const BARK_RADIUS_PX: f32 = 120.0;
const BARK_COOLDOWN_SECS: f32 = 20.0;

const SHEET_COLS: u32 = 8;
const SHEET_ROWS: u32 = 4;
const FRAME_SIZE_PX: u32 = 32;
const IDLE_FRAMES: usize = 4;
const WALK_FRAMES: usize = 4;

// ---------------------------------------------------------------------------
// Marker component
// ---------------------------------------------------------------------------

/// Shared marker for all area-event NPCs (not Galen, who is separate).
#[derive(Component)]
pub struct EventNpc;

// ---------------------------------------------------------------------------
// Spawn / despawn systems
// ---------------------------------------------------------------------------

pub fn despawn_npcs(mut commands: Commands, q: Query<Entity, With<EventNpc>>) {
    for entity in &q {
        commands.entity(entity).despawn();
    }
}

/// Spawn an NPC for the given area at its absolute world position.
/// Called from `spawning::ensure_area_spawned`.
pub fn spawn_npc_for_area(
    commands: &mut Commands,
    asset_server: &AssetServer,
    atlas_layouts: &mut Assets<TextureAtlasLayout>,
    area: &crate::area::Area,
    area_pos: IVec2,
) {
    let AreaEvent::NpcEncounter(npc_kind) = area.event else {
        return;
    };
    let base = crate::spawning::area_world_offset(area_pos);
    spawn_npc(commands, asset_server, atlas_layouts, npc_kind, base);
}

fn spawn_npc(
    commands: &mut Commands,
    asset_server: &AssetServer,
    atlas_layouts: &mut Assets<TextureAtlasLayout>,
    kind: NpcKind,
    base: Vec2,
) {
    let (name, sheet, script, barks) = npc_data(kind);
    let pos = tile_world_pos(PATH_CENTER_X, PATH_CENTER_Y, base);

    commands.spawn((
        EventNpc,
        Name::new(name),
        npc_sprite(asset_server, atlas_layouts, sheet),
        npc_collider(),
        Transform::from_translation(pos),
        npc_anim_bundle(pos.truncate()),
        Talker::new(asset_server.load(script)),
        bark_pool(asset_server, barks),
    ));
}

/// Returns (display_name, sheet_path, dialogue_script, bark_paths) for each NPC.
fn npc_data(kind: NpcKind) -> (&'static str, &'static str, &'static str, &'static [&'static str]) {
    match kind {
        NpcKind::Mordred => (
            "Mordred",
            "sprites/npc/npc_mordred_sheet.webp",
            "dialogue/scripts/mordred.dialog.ron",
            &[
                "dialogue/barks/mordred_barks.dialog.ron",
                "dialogue/barks/mordred_barks2.dialog.ron",
                "dialogue/barks/mordred_barks3.dialog.ron",
            ],
        ),
        NpcKind::Drizella => (
            "Drizella Tremaine",
            "sprites/npc/npc_drizella_sheet.webp",
            "dialogue/scripts/drizella.dialog.ron",
            &[
                "dialogue/barks/drizella_barks.dialog.ron",
                "dialogue/barks/drizella_barks2.dialog.ron",
                "dialogue/barks/drizella_barks3.dialog.ron",
            ],
        ),
        NpcKind::Bigby => (
            "Bigby",
            "sprites/npc/npc_bigby_sheet.webp",
            "dialogue/scripts/bigby.dialog.ron",
            &[
                "dialogue/barks/bigby_barks.dialog.ron",
                "dialogue/barks/bigby_barks2.dialog.ron",
                "dialogue/barks/bigby_barks3.dialog.ron",
            ],
        ),
        NpcKind::Gothel => (
            "Dame Gothel",
            "sprites/npc/npc_gothel_sheet.webp",
            "dialogue/scripts/mother_gothel.dialog.ron",
            &[
                "dialogue/barks/mother_gothel_barks.dialog.ron",
                "dialogue/barks/mother_gothel_barks2.dialog.ron",
                "dialogue/barks/mother_gothel_barks3.dialog.ron",
            ],
        ),
        NpcKind::Morgana => (
            "Morgana Le Fay",
            "sprites/npc/npc_morgana_sheet.webp",
            "dialogue/scripts/morgana.dialog.ron",
            &[
                "dialogue/barks/morgana_barks.dialog.ron",
                "dialogue/barks/morgana_barks2.dialog.ron",
                "dialogue/barks/morgana_barks3.dialog.ron",
            ],
        ),
        NpcKind::Cadwallader => (
            "Memphis Cadwallader",
            "sprites/npc/npc_cadwallader_sheet.webp",
            "dialogue/scripts/cadwallader.dialog.ron",
            &[
                "dialogue/barks/cadwallader_barks.dialog.ron",
                "dialogue/barks/cadwallader_barks2.dialog.ron",
                "dialogue/barks/cadwallader_barks3.dialog.ron",
            ],
        ),
    }
}

fn tile_world_pos(tx: u16, ty: u16, base: Vec2) -> Vec3 {
    let tile_px = f32::from(TILE_SIZE_PX);
    let offset_x = base.x - (f32::from(MAP_WIDTH) * tile_px) / 2.0;
    let offset_y = base.y - (f32::from(MAP_HEIGHT) * tile_px) / 2.0;
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
        NpcSheet {
            idle_frames: IDLE_FRAMES,
            walk_frames: WALK_FRAMES,
            cols: SHEET_COLS.try_into().expect("SHEET_COLS fits usize"),
        },
        NpcAnimFrame::default(),
        NpcAnimTimer::default(),
        NpcWander::new(origin),
    )
}
