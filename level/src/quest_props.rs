//! Spawn `AreaEvent::QuestProp` landmarks. Placement is decided by
//! [`crate::world::place_quest_props`] during world-gen; this module just
//! turns the picked variant into a Bevy entity at a deterministic spot
//! within the area.
//!
//! Sick deer carries a `Talker`; purple pond carries a quest-aware
//! `Investigatable` whose investigation triggers the mirror-shard choice.

use bevy::prelude::*;
use dialog::components::Talker;
use models::layer::Layer;
use quest::interact::{InvestigateChoice, Investigatable};
use quest::inventory::item;

use crate::area::{AreaEvent, QuestPropKind, MAP_HEIGHT, MAP_WIDTH};
use crate::spawning::{area_world_offset, TILE_SIZE_PX};

const Y_SORT_SCALE: f32 = 0.001;

const SICK_DEER_SPRITE: &str = "sprites/scenery/quest/sick_deer.webp";
const SICK_DEER_SIZE_PX: f32 = 32.0;
const SICK_DEER_SCRIPT: &str = "dialogue/scripts/sick_deer.dialog.ron";
/// Tile coordinate (x, y) within the area where the sick deer sits.
const SICK_DEER_TILE: (u16, u16) = (10, 6);

const PURPLE_POND_SPRITE: &str = "sprites/scenery/quest/purple_pond.webp";
const PURPLE_POND_SIZE_PX: f32 = 48.0;
/// Tile coordinate (x, y) within the area where the purple pond sits.
const PURPLE_POND_TILE: (u16, u16) = (20, 10);

const POND_DESCRIPTION: &str = "quest.bigby.sick_animals.pond.description";
const POND_FLAG: &str = "quest:bigby.sick_animals:milestone:1";

pub fn spawn_quest_prop_for_area(
    commands: &mut Commands,
    asset_server: &AssetServer,
    area: &crate::area::Area,
    area_pos: IVec2,
) {
    let AreaEvent::QuestProp(kind) = area.event else {
        return;
    };
    let base = area_world_offset(area_pos);
    match kind {
        QuestPropKind::SickDeer => spawn_sick_deer(commands, asset_server, base),
        QuestPropKind::PurplePond => spawn_purple_pond(commands, asset_server, base),
    }
}

#[derive(Component)]
pub struct QuestProp;

fn spawn_sick_deer(commands: &mut Commands, asset_server: &AssetServer, base: Vec2) {
    let pos = tile_world_pos(SICK_DEER_TILE.0, SICK_DEER_TILE.1, base);
    commands.spawn((
        QuestProp,
        Name::new("Sick Deer"),
        Sprite {
            image: asset_server.load(SICK_DEER_SPRITE),
            custom_size: Some(Vec2::splat(SICK_DEER_SIZE_PX)),
            ..default()
        },
        Transform::from_translation(pos),
        Talker::new(asset_server.load(SICK_DEER_SCRIPT)),
    ));
}

fn spawn_purple_pond(commands: &mut Commands, asset_server: &AssetServer, base: Vec2) {
    let pos = tile_world_pos(PURPLE_POND_TILE.0, PURPLE_POND_TILE.1, base);
    let investigatable = Investigatable::new(POND_DESCRIPTION, POND_FLAG).with_choices(vec![
        InvestigateChoice {
            text_key: "quest.bigby.sick_animals.shard.consume".to_string(),
            alignment_grant: Some(models::alignment::AlignmentFaction::Darkwoods),
            item_grant: None,
            flag_to_set: Some("quest:bigby.sick_animals:shard:consumed".to_string()),
            response_key: Some("quest.bigby.sick_animals.shard.consume_response".to_string()),
        },
        InvestigateChoice {
            text_key: "quest.bigby.sick_animals.shard.destroy".to_string(),
            alignment_grant: Some(models::alignment::AlignmentFaction::Greenwoods),
            item_grant: None,
            flag_to_set: Some("quest:bigby.sick_animals:shard:destroyed".to_string()),
            response_key: Some("quest.bigby.sick_animals.shard.destroy_response".to_string()),
        },
        InvestigateChoice {
            text_key: "quest.bigby.sick_animals.shard.pocket".to_string(),
            alignment_grant: None,
            item_grant: Some((item::MIRROR_SHARD.to_string(), 1)),
            flag_to_set: Some("quest:bigby.sick_animals:shard:pocketed".to_string()),
            response_key: Some("quest.bigby.sick_animals.shard.pocket_response".to_string()),
        },
    ]);
    commands.spawn((
        QuestProp,
        Name::new("Purple Pond"),
        Sprite {
            image: asset_server.load(PURPLE_POND_SPRITE),
            custom_size: Some(Vec2::splat(PURPLE_POND_SIZE_PX)),
            ..default()
        },
        Transform::from_translation(pos),
        investigatable,
    ));
}

pub fn despawn_quest_props(mut commands: Commands, q: Query<Entity, With<QuestProp>>) {
    for entity in &q {
        commands.entity(entity).despawn();
    }
}

fn tile_world_pos(tx: u16, ty: u16, base: Vec2) -> Vec3 {
    let tile_px = f32::from(TILE_SIZE_PX);
    let offset_x = base.x - (f32::from(MAP_WIDTH) * tile_px) / 2.0;
    let offset_y = base.y - (f32::from(MAP_HEIGHT) * tile_px) / 2.0;
    let world_y = offset_y + f32::from(ty) * tile_px + tile_px / 2.0;
    Vec3::new(
        offset_x + f32::from(tx) * tile_px + tile_px / 2.0,
        world_y,
        Layer::World.z_f32() - world_y * Y_SORT_SCALE,
    )
}
