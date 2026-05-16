//! Fixed quest entities spawned at world setup: the sick deer (milestone 0)
//! and the purple pond (milestone 1).
//!
//! These are one-off, hand-placed props -- not procedural creatures. The
//! positions are absolute world coordinates near the origin path
//! intersection so the player encounters them naturally on first play.

use bevy::prelude::*;
use dialog::components::Talker;
use models::layer::Layer;

use crate::interact::Investigatable;
use crate::inventory::item;

/// Where the sick deer sits relative to world origin.
const SICK_DEER_POS: Vec2 = Vec2::new(96.0, -64.0);
/// Where the purple pond sits relative to world origin.
const PURPLE_POND_POS: Vec2 = Vec2::new(-128.0, 96.0);

const SICK_DEER_SPRITE: &str = "sprites/scenery/quest/sick_deer.webp";
const PURPLE_POND_SPRITE: &str = "sprites/scenery/quest/purple_pond.webp";

const SICK_DEER_SIZE_PX: f32 = 32.0;
const PURPLE_POND_SIZE_PX: f32 = 48.0;
const Y_SORT_SCALE: f32 = 0.001;

const SICK_DEER_SCRIPT: &str = "dialogue/scripts/sick_deer.dialog.ron";

/// Marker for the sick deer entity (lets us despawn cleanly on world reload).
#[derive(Component)]
pub struct SickDeer;

/// Marker for the purple pond entity.
#[derive(Component)]
pub struct PurplePond;

pub fn spawn_quest_props(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        SickDeer,
        Name::new("Sick Deer"),
        Sprite {
            image: asset_server.load(SICK_DEER_SPRITE),
            custom_size: Some(Vec2::splat(SICK_DEER_SIZE_PX)),
            ..default()
        },
        Transform::from_xyz(
            SICK_DEER_POS.x,
            SICK_DEER_POS.y,
            Layer::World.z_f32() - SICK_DEER_POS.y * Y_SORT_SCALE,
        ),
        Talker::new(asset_server.load(SICK_DEER_SCRIPT)),
    ));

    commands.spawn((
        PurplePond,
        Name::new("Purple Pond"),
        Sprite {
            image: asset_server.load(PURPLE_POND_SPRITE),
            custom_size: Some(Vec2::splat(PURPLE_POND_SIZE_PX)),
            ..default()
        },
        Transform::from_xyz(
            PURPLE_POND_POS.x,
            PURPLE_POND_POS.y,
            Layer::World.z_f32() - PURPLE_POND_POS.y * Y_SORT_SCALE,
        ),
        Investigatable::new(
            "quest.bigby.sick_animals.pond.description",
            "quest:bigby.sick_animals:milestone:1",
        )
        .with_item(item::MIRROR_SHARD, 1),
    ));
}

pub fn despawn_quest_props(
    mut commands: Commands,
    deer_q: Query<Entity, With<SickDeer>>,
    pond_q: Query<Entity, With<PurplePond>>,
) {
    for entity in deer_q.iter().chain(pond_q.iter()) {
        commands.entity(entity).despawn();
    }
}
