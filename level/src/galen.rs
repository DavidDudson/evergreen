//! Storyteller Galen — the NPC who greets the player on spawn and poses
//! randomised hero alignment questions.

use bevy::prelude::*;
use dialog::asset::DialogueScript;
use dialog::components::{DialogueTrigger, Talker};
use models::layer::Layer;
use models::npc_anim::{NpcAnimFrame, NpcAnimKind, NpcAnimTimer, NpcFacing, NpcSheet};
use models::scenery::SceneryCollider;
use rand::seq::SliceRandom;

use bevy::math::IVec2;

use crate::area::{MAP_HEIGHT, MAP_WIDTH};
use crate::npc_wander::NpcWander;
use crate::spawning::TILE_SIZE_PX;
use crate::world::{AreaChanged, WorldMap};

/// Galen's home area (the starting area).
const AREA_GALEN: IVec2 = IVec2::new(0, 0);

// Galen stands on the N arm of the starting area (col 15, row 13).
// Tile (15, 13) is on columns 14-16 with y >= 7 (North exit guaranteed).
const GALEN_TILE_X: u16 = 15;
const GALEN_TILE_Y: u16 = 13;
const GALEN_Z: f32 = Layer::Npc.z_f32();
const GALEN_SPRITE_SIZE_PX: f32 = 32.0;
const GALEN_COLLIDER_HALF: Vec2 = Vec2::new(7.0, 7.0);
const GALEN_QUESTION_COUNT: usize = 5;

// Sprite sheet layout: 4 rows (S/E/N/W) × 8 cols (4 idle + 4 walk).
const SHEET_COLS: u32 = 8;
const SHEET_ROWS: u32 = 4;
const FRAME_SIZE_PX: u32 = 32;
const IDLE_FRAMES: usize = 4;
const WALK_FRAMES: usize = 4;

/// Marker for the Storyteller Galen entity.
#[derive(Component)]
pub struct NpcGalen;

/// Attached to Galen; holds all hero-question script handles to choose from.
#[derive(Component)]
pub struct GalenQuestioner {
    pub questions: Vec<Handle<DialogueScript>>,
}

pub fn spawn_galen(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    world: Res<WorldMap>,
    existing: Query<(), With<NpcGalen>>,
) {
    if !existing.is_empty() || world.current != AREA_GALEN {
        return;
    }
    spawn_galen_entity(&mut commands, &asset_server, &mut atlas_layouts);
}

fn spawn_galen_entity(
    commands: &mut Commands,
    asset_server: &AssetServer,
    atlas_layouts: &mut Assets<TextureAtlasLayout>,
) {
    let layout = TextureAtlasLayout::from_grid(
        UVec2::splat(FRAME_SIZE_PX),
        SHEET_COLS,
        SHEET_ROWS,
        None,
        None,
    );
    let layout_handle = atlas_layouts.add(layout);
    let pos = galen_pos();
    let questions: Vec<Handle<DialogueScript>> = (1..=GALEN_QUESTION_COUNT)
        .map(|i| asset_server.load(format!("dialogue/scripts/galen_q{i}.dialog.ron")))
        .collect();
    let initial = questions[0].clone();
    commands.spawn((
        NpcGalen,
        Name::new("Storyteller Galen"),
        Sprite {
            image: asset_server.load("npc_galen_sheet.png"),
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
        Talker::repeating(initial),
        GalenQuestioner { questions },
    ));
}

fn galen_pos() -> Vec3 {
    let tile_px = f32::from(TILE_SIZE_PX);
    let offset_x = -(f32::from(MAP_WIDTH) * tile_px) / 2.0;
    let offset_y = -(f32::from(MAP_HEIGHT) * tile_px) / 2.0;
    Vec3::new(
        offset_x + f32::from(GALEN_TILE_X) * tile_px + tile_px / 2.0,
        offset_y + f32::from(GALEN_TILE_Y) * tile_px + tile_px / 2.0,
        GALEN_Z,
    )
}

pub fn despawn_galen(mut commands: Commands, q: Query<Entity, With<NpcGalen>>) {
    for entity in &q {
        commands.entity(entity).despawn();
    }
}

/// Despawns and conditionally respawns Galen when the player changes area.
pub fn respawn_galen_on_area_change(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    world: Res<WorldMap>,
    q: Query<Entity, With<NpcGalen>>,
    mut events: MessageReader<AreaChanged>,
) {
    if events.read().next().is_none() {
        return;
    }
    for entity in &q {
        commands.entity(entity).despawn();
    }
    if world.current == AREA_GALEN {
        spawn_galen_entity(&mut commands, &asset_server, &mut atlas_layouts);
    }
}

/// Randomises Galen's greeting script each time a player enters his interact range.
///
/// Uses [`Added<DialogueTrigger>`] to detect new approach events.
pub fn randomise_question(
    trigger_q: Query<&DialogueTrigger, Added<DialogueTrigger>>,
    mut galen_q: Query<(&mut Talker, &GalenQuestioner), With<NpcGalen>>,
) {
    let Ok(trigger) = trigger_q.single() else {
        return;
    };
    let Ok((mut talker, questioner)) = galen_q.get_mut(trigger.npc) else {
        return;
    };
    let mut rng = rand::thread_rng();
    if let Some(handle) = questioner.questions.choose(&mut rng) {
        talker.greeting = handle.clone();
    }
}
