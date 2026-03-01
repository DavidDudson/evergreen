use bevy::prelude::*;
use level::plugin::tile_size;
use models::layer::Layer;
use models::speed::Speed;
use models::tile::Tile;

use crate::animation::{
    AnimationFrame, AnimationKind, AnimationTimer, FacingDirection, FRAME_H_PX, FRAME_W_PX,
    SHEET_COLS, SHEET_ROWS,
};

const PLAYER_WIDTH: Tile = Tile(1);
const PLAYER_HEIGHT: Tile = Tile(2);
pub const PLAYER_SPEED: Speed = Speed(6); // run speed: 6 tiles/s; walk is 2 tiles/s (see movement.rs)

#[derive(Component)]
#[require(Speed)]
pub struct Player;

pub fn spawn(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    let layout = TextureAtlasLayout::from_grid(
        UVec2::new(FRAME_W_PX, FRAME_H_PX),
        u32::try_from(SHEET_COLS).expect("SHEET_COLS fits u32"),
        u32::try_from(SHEET_ROWS).expect("SHEET_ROWS fits u32"),
        None,
        None,
    );
    let layout_handle = atlas_layouts.add(layout);

    commands.spawn((
        Player,
        PLAYER_SPEED,
        FacingDirection::default(),
        AnimationKind::default(),
        AnimationFrame::default(),
        AnimationTimer::default(),
        Sprite {
            image: asset_server.load("briar_sheet.png"),
            texture_atlas: Some(TextureAtlas {
                layout: layout_handle,
                index: 0,
            }),
            custom_size: Some(tile_size(PLAYER_WIDTH, PLAYER_HEIGHT)),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, Layer::Player.z_f32()),
    ));
}

pub fn despawn(mut commands: Commands, query: Query<Entity, With<Player>>) {
    query
        .iter()
        .for_each(|entity| commands.entity(entity).despawn());
}
