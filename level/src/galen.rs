//! Storyteller Galen -- the NPC who greets the player on spawn and poses
//! a single randomised hero alignment question. After the first conversation,
//! Galen switches to barks.

use bevy::prelude::*;
use dialog::asset::DialogueScript;
use dialog::components::{BarkPool, Talker};
use models::layer::Layer;
use models::npc_anim::{NpcAnimFrame, NpcAnimKind, NpcAnimTimer, NpcFacing, NpcSheet};
use models::scenery::SceneryCollider;
use rand::seq::IndexedRandom;

use crate::area::{MAP_HEIGHT, MAP_WIDTH};
use crate::npc_wander::NpcWander;
use crate::shadows::{spawn_drop_shadow, DropShadowAssets};
use crate::spawning::TILE_SIZE_PX;
use crate::world::WorldMap;
use models::shadow::{GALEN_SHADOW_HALF_PX, GALEN_SHADOW_OFFSET_Y_PX};

// Galen stands on the N arm of the starting area (col 15, row 13).
const GALEN_TILE_X: u16 = 15;
const GALEN_TILE_Y: u16 = 13;
const Y_SORT_SCALE: f32 = 0.001;
const GALEN_SPRITE_SIZE_PX: f32 = 32.0;
const GALEN_COLLIDER_HALF: Vec2 = Vec2::new(7.0, 7.0);
const GALEN_QUESTION_COUNT: usize = 5;

// Sprite sheet layout: 4 rows (S/E/N/W) x 8 cols (4 idle + 4 walk).
const SHEET_COLS: u32 = 8;
const SHEET_ROWS: u32 = 4;
const FRAME_SIZE_PX: u32 = 32;
const IDLE_FRAMES: usize = 4;
const WALK_FRAMES: usize = 4;

const BARK_RADIUS_PX: f32 = 120.0;
const BARK_COOLDOWN_SECS: f32 = 20.0;

/// Marker for the Storyteller Galen entity.
#[derive(Component)]
pub struct NpcGalen;

pub fn spawn_galen(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    shadow_assets: Res<DropShadowAssets>,
    world: Res<WorldMap>,
    existing: Query<(), With<NpcGalen>>,
) {
    if !existing.is_empty() {
        return;
    }
    spawn_galen_entity(
        &mut commands,
        &asset_server,
        &mut atlas_layouts,
        &shadow_assets,
        world.current,
    );
}

fn spawn_galen_entity(
    commands: &mut Commands,
    asset_server: &AssetServer,
    atlas_layouts: &mut Assets<TextureAtlasLayout>,
    shadow_assets: &DropShadowAssets,
    start_area: IVec2,
) {
    let layout = TextureAtlasLayout::from_grid(
        UVec2::splat(FRAME_SIZE_PX),
        SHEET_COLS,
        SHEET_ROWS,
        None,
        None,
    );
    let layout_handle = atlas_layouts.add(layout);
    let pos = galen_pos(start_area);

    // Pick one random question from the pool.
    let questions: Vec<Handle<DialogueScript>> = (1..=GALEN_QUESTION_COUNT)
        .map(|i| asset_server.load(format!("dialogue/scripts/galen_q{i}.dialog.ron")))
        .collect();
    let mut rng = rand::rng();
    let chosen = questions
        .choose(&mut rng)
        .expect("question pool is non-empty")
        .clone();

    let parent = commands.spawn((
        NpcGalen,
        Name::new("Storyteller Galen"),
        Sprite {
            image: asset_server.load("sprites/npc/npc_galen_sheet.webp"),
            texture_atlas: Some(TextureAtlas {
                layout: layout_handle,
                index: 0,
            }),
            custom_size: Some(Vec2::splat(GALEN_SPRITE_SIZE_PX)),
            ..default()
        },
        SceneryCollider {
            half_extents: GALEN_COLLIDER_HALF,
            center_offset: Vec2::ZERO,
        },
        Transform::from_translation(pos),
        NpcFacing::default(),
        NpcAnimKind::default(),
        NpcSheet {
            idle_frames: IDLE_FRAMES,
            walk_frames: WALK_FRAMES,
            cols: SHEET_COLS.try_into().expect("SHEET_COLS fits usize"),
        },
        NpcAnimFrame::default(),
        NpcAnimTimer::default(),
        NpcWander::new(pos.truncate()),
        // Non-repeating: Galen only asks one question, then stops offering dialogue.
        Talker::new(chosen),
        BarkPool {
            barks: vec![
                asset_server.load("dialogue/barks/galen_bark1.dialog.ron"),
                asset_server.load("dialogue/barks/galen_bark2.dialog.ron"),
                asset_server.load("dialogue/barks/galen_bark3.dialog.ron"),
            ],
            trigger_radius_px: BARK_RADIUS_PX,
            cooldown: Timer::from_seconds(BARK_COOLDOWN_SECS, TimerMode::Once),
        },
    )).id();

    spawn_drop_shadow(commands, shadow_assets, parent, GALEN_SHADOW_HALF_PX, GALEN_SHADOW_OFFSET_Y_PX);
}

fn galen_pos(start_area: IVec2) -> Vec3 {
    let tile_px = f32::from(TILE_SIZE_PX);
    let base = crate::spawning::area_world_offset(start_area);
    let offset_x = base.x - (f32::from(MAP_WIDTH) * tile_px) / 2.0;
    let offset_y = base.y - (f32::from(MAP_HEIGHT) * tile_px) / 2.0;
    let world_y = offset_y + f32::from(GALEN_TILE_Y) * tile_px + tile_px / 2.0;
    Vec3::new(
        offset_x + f32::from(GALEN_TILE_X) * tile_px + tile_px / 2.0,
        world_y,
        Layer::World.z_f32() - world_y * Y_SORT_SCALE,
    )
}

/// Update Galen's z-position for y-sort ordering.
pub fn update_galen_z(mut query: Query<&mut Transform, With<NpcGalen>>) {
    let Ok(mut tf) = query.single_mut() else {
        return;
    };
    tf.translation.z = Layer::World.z_f32() - tf.translation.y * Y_SORT_SCALE;
}

pub fn despawn_galen(mut commands: Commands, q: Query<Entity, With<NpcGalen>>) {
    for entity in &q {
        commands.entity(entity).despawn();
    }
}
