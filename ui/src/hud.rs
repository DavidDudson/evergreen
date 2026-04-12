use bevy::prelude::*;
use models::alignment::{AlignmentFaction, PlayerAlignment};
use models::health::Health;
use models::palette;
use std::f32::consts::TAU;

use crate::fonts::UiFont;

// ---------------------------------------------------------------------------
// Rose petal health constants
// ---------------------------------------------------------------------------

const HUD_OFFSET_PX: f32 = 8.0;
const ROSE_CONTAINER_PX: f32 = 80.0;
const PETAL_SIZE_PX: f32 = 32.0;
const PETAL_ORBIT_RADIUS_PX: f32 = 20.0;
const PETAL_COUNT: u16 = 10;

// ---------------------------------------------------------------------------
// Alignment bar constants
// ---------------------------------------------------------------------------

const ALIGN_OFFSET_PX: f32 = 8.0;
const ALIGN_BAR_WIDTH_PX: f32 = 100.0;
const ALIGN_BAR_HEIGHT_PX: f32 = 8.0;
const ALIGN_ROW_GAP_PX: f32 = 6.0;
const ALIGN_LABEL_FONT_PX: f32 = 10.0;
const ALIGN_LABEL_WIDTH_PX: f32 = 72.0;
const ALIGN_COL_GAP_PX: f32 = 4.0;
const ALIGN_MAX: f32 = 10.0;

// ---------------------------------------------------------------------------
// Components
// ---------------------------------------------------------------------------

#[derive(Component)]
pub struct Hud;

#[derive(Component)]
pub struct RosePetal(pub u16);

#[derive(Component)]
pub struct AlignmentBars;

#[derive(Component)]
pub struct AlignmentBarFill(pub AlignmentFaction);

// ---------------------------------------------------------------------------
// Setup
// ---------------------------------------------------------------------------

pub fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let petal_image = asset_server.load("sprites/ui/rose_petal.webp");
    let half_petal = PETAL_SIZE_PX / 2.0;
    let center = ROSE_CONTAINER_PX / 2.0;

    commands
        .spawn((
            Hud,
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(HUD_OFFSET_PX),
                right: Val::Px(HUD_OFFSET_PX),
                width: Val::Px(ROSE_CONTAINER_PX),
                height: Val::Px(ROSE_CONTAINER_PX),
                ..Node::default()
            },
        ))
        .with_children(|parent| {
            for i in 1..=PETAL_COUNT {
                let angle = TAU * f32::from(i - 1) / f32::from(PETAL_COUNT);
                let x = PETAL_ORBIT_RADIUS_PX * angle.sin();
                let y = -PETAL_ORBIT_RADIUS_PX * angle.cos();

                parent.spawn((
                    RosePetal(i),
                    ImageNode::new(petal_image.clone()),
                    Node {
                        position_type: PositionType::Absolute,
                        left: Val::Px(center + x - half_petal),
                        top: Val::Px(center + y - half_petal),
                        width: Val::Px(PETAL_SIZE_PX),
                        height: Val::Px(PETAL_SIZE_PX),
                        ..Node::default()
                    },
                    Transform::from_rotation(Quat::from_rotation_z(-angle)),
                ));
            }
        });
}

pub fn setup_alignment_bars(
    mut commands: Commands,
    alignment: Res<PlayerAlignment>,
    fonts: Res<UiFont>,
) {
    let root = commands
        .spawn((
            AlignmentBars,
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(ALIGN_OFFSET_PX),
                left: Val::Px(ALIGN_OFFSET_PX),
                display: Display::Flex,
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(ALIGN_ROW_GAP_PX),
                ..Node::default()
            },
        ))
        .id();

    for faction in [
        AlignmentFaction::Greenwoods,
        AlignmentFaction::Darkwoods,
        AlignmentFaction::Cities,
    ] {
        spawn_alignment_row(&mut commands, root, faction, alignment.get(faction), fonts.0.clone());
    }
}

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

pub fn sync_petals(
    health_query: Query<&Health, Changed<Health>>,
    petal_query: Query<(Entity, &RosePetal)>,
    mut commands: Commands,
) {
    let Ok(health) = health_query.single() else {
        return;
    };
    petal_query
        .iter()
        .filter(|(_, petal)| petal.0 > health.0)
        .for_each(|(entity, _)| commands.entity(entity).despawn());
}

pub fn sync_alignment_bars(
    alignment: Res<PlayerAlignment>,
    mut fill_q: Query<(&AlignmentBarFill, &mut Node)>,
) {
    if !alignment.is_changed() {
        return;
    }
    for (bar, mut node) in &mut fill_q {
        let score = alignment.get(bar.0);
        node.width = Val::Percent(f32::from(score) / ALIGN_MAX * 100.0);
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn spawn_alignment_row(
    commands: &mut Commands,
    parent: Entity,
    faction: AlignmentFaction,
    score: u8,
    font: Handle<Font>,
) {
    let (label, fill_color) = faction_display(faction);
    let fill_pct = f32::from(score) / ALIGN_MAX * 100.0;

    let row = commands
        .spawn((
            Node {
                display: Display::Flex,
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: Val::Px(ALIGN_COL_GAP_PX),
                ..Node::default()
            },
            ChildOf(parent),
        ))
        .id();

    commands.spawn((
        Text::new(label),
        TextFont { font, font_size: ALIGN_LABEL_FONT_PX, ..default() },
        TextColor(palette::ALIGN_LABEL),
        Node {
            width: Val::Px(ALIGN_LABEL_WIDTH_PX),
            ..Node::default()
        },
        ChildOf(row),
    ));

    let track = commands
        .spawn((
            Node {
                width: Val::Px(ALIGN_BAR_WIDTH_PX),
                height: Val::Px(ALIGN_BAR_HEIGHT_PX),
                overflow: Overflow::clip(),
                ..Node::default()
            },
            BackgroundColor(palette::ALIGN_TRACK),
            ChildOf(row),
        ))
        .id();

    commands.spawn((
        AlignmentBarFill(faction),
        Node {
            width: Val::Percent(fill_pct),
            height: Val::Percent(100.0),
            ..Node::default()
        },
        BackgroundColor(fill_color),
        ChildOf(track),
    ));
}

fn faction_display(faction: AlignmentFaction) -> (&'static str, Color) {
    match faction {
        AlignmentFaction::Greenwoods => ("Greenwoods", palette::ALIGN_GREENWOODS),
        AlignmentFaction::Darkwoods => ("Darkwoods", palette::ALIGN_DARKWOODS),
        AlignmentFaction::Cities => ("Cities", palette::ALIGN_CITIES),
    }
}
