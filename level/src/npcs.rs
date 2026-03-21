//! NPC entity spawning.
//!
//! Each NPC is a placeholder entity with a [`Talker`] component pointing to its
//! greeting script and a [`BarkPool`] of ambient lines.  Positions are
//! approximate world-space pixels and should be adjusted once actual map
//! layout is finalised.
//!
//! Characters covered:
//!  - Mordred               (Mythmakers Union PC — NPC in Evergreen)
//!  - Drizella Tremaine     (Mythmakers Union PC — NPC in Evergreen)
//!  - Bigby                 (Mythmakers Union PC — NPC in Evergreen)
//!  - Dame Gothel           (former All-Mother of the Thrice-Triple Covens)
//!  - Morgana Le Fay        (The Begot Queen; Mordred's adoptive mother)
//!  - Memphis Cadwallader   (The Imp / Rumpelstiltskin; deal-maker)

use bevy::prelude::*;
use dialog::components::{BarkPool, Talker};

use crate::spawning::TILE_SIZE_PX;

// ---------------------------------------------------------------------------
// Placeholder world positions (tile coordinates → pixels)
// Adjust freely once the world map is designed.
// ---------------------------------------------------------------------------

const MORDRED_TILE_X: i32 = 30;
const MORDRED_TILE_Y: i32 = 20;

const DRIZELLA_TILE_X: i32 = -15;
const DRIZELLA_TILE_Y: i32 = 10;

const BIGBY_TILE_X: i32 = 10;
const BIGBY_TILE_Y: i32 = -8;

const GOTHEL_TILE_X: i32 = -40;
const GOTHEL_TILE_Y: i32 = 35;

const MORGANA_TILE_X: i32 = -60;
const MORGANA_TILE_Y: i32 = 50;

const CADWALLADER_TILE_X: i32 = 5;
const CADWALLADER_TILE_Y: i32 = 5;

const NPC_Z: f32 = 1.0;
const BARK_RADIUS_PX: f32 = 120.0;
const BARK_COOLDOWN_SECS: f32 = 20.0;

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
// Spawn system
// ---------------------------------------------------------------------------

pub fn spawn_npcs(mut commands: Commands, asset_server: Res<AssetServer>) {
    spawn_mordred(&mut commands, &asset_server);
    spawn_drizella(&mut commands, &asset_server);
    spawn_bigby(&mut commands, &asset_server);
    spawn_gothel(&mut commands, &asset_server);
    spawn_morgana(&mut commands, &asset_server);
    spawn_cadwallader(&mut commands, &asset_server);
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

// ---------------------------------------------------------------------------
// Per-character helpers
// ---------------------------------------------------------------------------

fn tile_pos(tile_x: i32, tile_y: i32) -> Vec3 {
    Vec3::new(
        tile_x as f32 * f32::from(TILE_SIZE_PX),
        tile_y as f32 * f32::from(TILE_SIZE_PX),
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

fn spawn_mordred(commands: &mut Commands, asset_server: &AssetServer) {
    commands.spawn((
        NpcMordred,
        Name::new("Mordred"),
        Transform::from_translation(tile_pos(MORDRED_TILE_X, MORDRED_TILE_Y)),
        Visibility::default(),
        Talker::new(asset_server.load("dialogue/scripts/mordred.dialog.ron")),
        bark_pool(asset_server, &[
            "dialogue/barks/mordred_barks.dialog.ron",
            "dialogue/barks/mordred_barks2.dialog.ron",
            "dialogue/barks/mordred_barks3.dialog.ron",
        ]),
    ));
}

fn spawn_drizella(commands: &mut Commands, asset_server: &AssetServer) {
    commands.spawn((
        NpcDrizella,
        Name::new("Drizella Tremaine"),
        Transform::from_translation(tile_pos(DRIZELLA_TILE_X, DRIZELLA_TILE_Y)),
        Visibility::default(),
        Talker::new(asset_server.load("dialogue/scripts/drizella.dialog.ron")),
        bark_pool(asset_server, &[
            "dialogue/barks/drizella_barks.dialog.ron",
            "dialogue/barks/drizella_barks2.dialog.ron",
            "dialogue/barks/drizella_barks3.dialog.ron",
        ]),
    ));
}

fn spawn_bigby(commands: &mut Commands, asset_server: &AssetServer) {
    commands.spawn((
        NpcBigby,
        Name::new("Bigby"),
        Transform::from_translation(tile_pos(BIGBY_TILE_X, BIGBY_TILE_Y)),
        Visibility::default(),
        Talker::new(asset_server.load("dialogue/scripts/bigby.dialog.ron")),
        bark_pool(asset_server, &[
            "dialogue/barks/bigby_barks.dialog.ron",
            "dialogue/barks/bigby_barks2.dialog.ron",
            "dialogue/barks/bigby_barks3.dialog.ron",
        ]),
    ));
}

fn spawn_gothel(commands: &mut Commands, asset_server: &AssetServer) {
    commands.spawn((
        NpcGothel,
        Name::new("Dame Gothel"),
        Transform::from_translation(tile_pos(GOTHEL_TILE_X, GOTHEL_TILE_Y)),
        Visibility::default(),
        Talker::new(asset_server.load("dialogue/scripts/mother_gothel.dialog.ron")),
        bark_pool(asset_server, &[
            "dialogue/barks/mother_gothel_barks.dialog.ron",
            "dialogue/barks/mother_gothel_barks2.dialog.ron",
            "dialogue/barks/mother_gothel_barks3.dialog.ron",
        ]),
    ));
}

fn spawn_morgana(commands: &mut Commands, asset_server: &AssetServer) {
    commands.spawn((
        NpcMorgana,
        Name::new("Morgana Le Fay"),
        Transform::from_translation(tile_pos(MORGANA_TILE_X, MORGANA_TILE_Y)),
        Visibility::default(),
        Talker::new(asset_server.load("dialogue/scripts/morgana.dialog.ron")),
        bark_pool(asset_server, &[
            "dialogue/barks/morgana_barks.dialog.ron",
            "dialogue/barks/morgana_barks2.dialog.ron",
            "dialogue/barks/morgana_barks3.dialog.ron",
        ]),
    ));
}

fn spawn_cadwallader(commands: &mut Commands, asset_server: &AssetServer) {
    commands.spawn((
        NpcCadwallader,
        Name::new("Memphis Cadwallader"),
        Transform::from_translation(tile_pos(CADWALLADER_TILE_X, CADWALLADER_TILE_Y)),
        Visibility::default(),
        Talker::new(asset_server.load("dialogue/scripts/cadwallader.dialog.ron")),
        bark_pool(asset_server, &[
            "dialogue/barks/cadwallader_barks.dialog.ron",
            "dialogue/barks/cadwallader_barks2.dialog.ron",
            "dialogue/barks/cadwallader_barks3.dialog.ron",
        ]),
    ));
}
