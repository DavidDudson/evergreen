//! Heads-up minimap.
//!
//! Layout follows web-design conventions:
//!   - rounded panel with a subtle drop shadow + 1px accent border
//!   - small caption header (`MAP n · ALN x`)
//!   - 5x5 grid of square cells with thin connectors
//!   - icon legend at the bottom (NPC / Enemy / Portal)
//!
//! All sizes / colors come from `models::tailwind` design tokens.

use bevy::math::IVec2;
use bevy::prelude::*;
use level::area::{AreaEvent, Direction};
use level::world::WorldMap;
use models::palette;
use models::tailwind as tw;

use crate::fonts::UiFont;

// ---------------------------------------------------------------------------
// Layout constants (Tailwind v4 spacing + radius)
// ---------------------------------------------------------------------------

const VIEW_RADIUS: i32 = 2;
const VIEW_SIZE: u16 = 5; // VIEW_RADIUS * 2 + 1

const CELL_PX: u16 = 16; // SPACE_4
const GAP_PX: u16 = 4; // SPACE_1
const CONNECTOR_THICKNESS_PX: u16 = 3;
const CELL_RADIUS_PX: f32 = tw::RADIUS_XS_PX;

const STEP_PX: u16 = CELL_PX + GAP_PX;
const GRID_PX: u16 = VIEW_SIZE * STEP_PX - GAP_PX;

const PANEL_PADDING_PX: f32 = tw::SPACE_3_PX;
const PANEL_RADIUS_PX: f32 = tw::RADIUS_LG_PX;
const PANEL_BORDER_PX: f32 = 1.0;
/// Width of the panel's content area (grid + header + legend share this).
const PANEL_CONTENT_W_PX: u16 = GRID_PX;

const HEADER_GAP_PX: f32 = tw::SPACE_2_PX;
const SECTION_GAP_PX: f32 = tw::SPACE_2_PX;

const SHADOW_OFFSET_PX: f32 = 3.0;

const TOP_OFFSET_PX: f32 = tw::SPACE_6_PX;
const RIGHT_OFFSET_PX: f32 = tw::SPACE_2_PX;

// Event-marker visuals.
const EVENT_DOT_PX: u16 = 5;
const EVENT_ICON_PX: u16 = 10;
const PORTAL_ICON_PATH: &str = "sprites/icons/portal.webp";
const ENEMY_ICON_PATH: &str = "sprites/icons/enemy.webp";

// Legend dot/icon size.
const LEGEND_DOT_PX: u16 = 7;
const LEGEND_ROW_GAP_PX: f32 = tw::SPACE_1_PX;
const LEGEND_ITEM_GAP_PX: f32 = tw::SPACE_1_5_PX;

// ---------------------------------------------------------------------------
// Components
// ---------------------------------------------------------------------------

#[derive(Component)]
pub struct MinimapRoot;

/// Marker on the grid container so refresh can rebuild only the cells.
#[derive(Component)]
pub struct MinimapGrid;

/// Marker on header text so refresh can update just the caption.
#[derive(Component)]
pub struct MinimapHeader;

/// Marker for cell + connector entities (children of the grid).
#[derive(Component)]
pub struct MinimapCell;

// ---------------------------------------------------------------------------
// Setup / despawn / refresh
// ---------------------------------------------------------------------------

pub fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    fonts: Res<UiFont>,
    world: Res<WorldMap>,
) {
    let shadow = commands
        .spawn((
            MinimapRoot,
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(TOP_OFFSET_PX + SHADOW_OFFSET_PX),
                right: Val::Px(RIGHT_OFFSET_PX - SHADOW_OFFSET_PX / 2.0),
                ..Node::default()
            },
            BackgroundColor(palette::TRANSPARENT),
        ))
        .id();

    // Drop-shadow layer: same shape as panel, offset diagonally, faded.
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(SHADOW_OFFSET_PX),
            left: Val::Px(SHADOW_OFFSET_PX),
            width: Val::Px(f32::from(PANEL_CONTENT_W_PX) + PANEL_PADDING_PX * 2.0),
            height: Val::Auto,
            border_radius: BorderRadius::all(Val::Px(PANEL_RADIUS_PX)),
            ..Node::default()
        },
        BackgroundColor(palette::PANEL_SHADOW),
        ChildOf(shadow),
    ));

    // Main panel.
    let panel = commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(0.0),
                left: Val::Px(0.0),
                width: Val::Px(f32::from(PANEL_CONTENT_W_PX) + PANEL_PADDING_PX * 2.0),
                padding: UiRect::all(Val::Px(PANEL_PADDING_PX)),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(SECTION_GAP_PX),
                border: UiRect::all(Val::Px(PANEL_BORDER_PX)),
                border_radius: BorderRadius::all(Val::Px(PANEL_RADIUS_PX)),
                ..Node::default()
            },
            BackgroundColor(palette::MINIMAP_BG),
            BorderColor::all(tw::EMERALD_700),
            ChildOf(shadow),
        ))
        .id();

    // Header row -- title label on left, map id on right.
    let header = commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                column_gap: Val::Px(HEADER_GAP_PX),
                ..Node::default()
            },
            ChildOf(panel),
        ))
        .id();
    commands.spawn((
        Text::new("MAP"),
        TextFont {
            font: fonts.0.clone(),
            font_size: tw::TEXT_XS_PX,
            ..default()
        },
        TextColor(tw::EMERALD_300),
        ChildOf(header),
    ));
    commands.spawn((
        MinimapHeader,
        Text::new(header_caption(&world)),
        TextFont {
            font: fonts.0.clone(),
            font_size: tw::TEXT_XS_PX,
            ..default()
        },
        TextColor(tw::EMERALD_200),
        ChildOf(header),
    ));

    // Divider.
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Px(1.0),
            ..Node::default()
        },
        BackgroundColor(tw::EMERALD_800),
        ChildOf(panel),
    ));

    // Grid container -- fixed size, cells positioned absolutely inside it.
    let grid = commands
        .spawn((
            MinimapGrid,
            Node {
                position_type: PositionType::Relative,
                width: Val::Px(f32::from(GRID_PX)),
                height: Val::Px(f32::from(GRID_PX)),
                ..Node::default()
            },
            ChildOf(panel),
        ))
        .id();

    build_grid_cells(grid, &asset_server, &world, &mut commands);

    // Divider.
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Px(1.0),
            ..Node::default()
        },
        BackgroundColor(tw::EMERALD_800),
        ChildOf(panel),
    ));

    // Legend.
    spawn_legend(panel, &asset_server, &fonts, &mut commands);
}

pub fn despawn(mut commands: Commands, root_q: Query<Entity, With<MinimapRoot>>) {
    for entity in &root_q {
        commands.entity(entity).despawn();
    }
}

/// Rebuild the grid cells + refresh the header caption when the world mutates.
pub fn refresh(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    world: Res<WorldMap>,
    grid_q: Query<Entity, With<MinimapGrid>>,
    cell_q: Query<Entity, With<MinimapCell>>,
    mut header_q: Query<&mut Text, With<MinimapHeader>>,
) {
    if !world.is_changed() {
        return;
    }
    let Ok(grid) = grid_q.single() else { return };

    for entity in &cell_q {
        commands.entity(entity).despawn();
    }

    build_grid_cells(grid, &asset_server, &world, &mut commands);

    if let Ok(mut text) = header_q.single_mut() {
        *text = Text::new(header_caption(&world));
    }
}

// ---------------------------------------------------------------------------
// Internals
// ---------------------------------------------------------------------------

fn header_caption(world: &WorldMap) -> String {
    format!("#{:02} · ALN {:>3}", world.id.0, world.alignment)
}

fn build_grid_cells(
    grid: Entity,
    asset_server: &AssetServer,
    world: &WorldMap,
    commands: &mut Commands,
) {
    let current = world.current;

    for dy in -VIEW_RADIUS..=VIEW_RADIUS {
        for dx in -VIEW_RADIUS..=VIEW_RADIUS {
            // Flip dy: screen y grows downward, world y grows northward.
            let area_pos = current + IVec2::new(dx, -dy);
            if !world.is_revealed(area_pos) {
                continue;
            }
            let Some(area) = world.get_area(area_pos) else {
                continue;
            };

            let is_current = area_pos == current;
            let cell_color = if is_current {
                tw::AMBER_400
            } else {
                tw::EMERALD_800
            };

            let col = u16::try_from(dx + VIEW_RADIUS).unwrap_or(0);
            let row = u16::try_from(dy + VIEW_RADIUS).unwrap_or(0);
            let cell_left = col * STEP_PX;
            let cell_top = row * STEP_PX;

            let cell_entity = commands
                .spawn((
                    MinimapCell,
                    Node {
                        position_type: PositionType::Absolute,
                        left: Val::Px(f32::from(cell_left)),
                        top: Val::Px(f32::from(cell_top)),
                        width: Val::Px(f32::from(CELL_PX)),
                        height: Val::Px(f32::from(CELL_PX)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        border_radius: BorderRadius::all(Val::Px(CELL_RADIUS_PX)),
                        ..Node::default()
                    },
                    BackgroundColor(cell_color),
                    ChildOf(grid),
                ))
                .id();

            spawn_cell_marker(commands, asset_server, world, area, area_pos, cell_entity);

            for &dir in &area.exits {
                if let Some((l, t, w, h)) = connector_rect(dir, cell_left, cell_top) {
                    commands.spawn((
                        MinimapCell,
                        Node {
                            position_type: PositionType::Absolute,
                            left: Val::Px(f32::from(l)),
                            top: Val::Px(f32::from(t)),
                            width: Val::Px(f32::from(w)),
                            height: Val::Px(f32::from(h)),
                            border_radius: BorderRadius::all(Val::Px(tw::RADIUS_XS_PX)),
                            ..Node::default()
                        },
                        BackgroundColor(tw::EMERALD_600),
                        ChildOf(grid),
                    ));
                }
            }
        }
    }
}

fn spawn_cell_marker(
    commands: &mut Commands,
    asset_server: &AssetServer,
    world: &WorldMap,
    area: &level::area::Area,
    area_pos: IVec2,
    cell: Entity,
) {
    let is_portal_area = world
        .portal
        .as_ref()
        .is_some_and(|p| p.area_pos == area_pos);

    if is_portal_area {
        commands.spawn((
            MinimapCell,
            Node {
                width: Val::Px(f32::from(EVENT_ICON_PX)),
                height: Val::Px(f32::from(EVENT_ICON_PX)),
                ..Node::default()
            },
            ImageNode::new(asset_server.load(PORTAL_ICON_PATH)),
            ChildOf(cell),
        ));
        return;
    }

    if matches!(area.event, AreaEvent::Enemy { .. }) {
        commands.spawn((
            MinimapCell,
            Node {
                width: Val::Px(f32::from(EVENT_ICON_PX)),
                height: Val::Px(f32::from(EVENT_ICON_PX)),
                ..Node::default()
            },
            ImageNode::new(asset_server.load(ENEMY_ICON_PATH)),
            ChildOf(cell),
        ));
        return;
    }

    if let Some(dot_color) = npc_dot_color(&area.event, area_pos) {
        commands.spawn((
            MinimapCell,
            Node {
                width: Val::Px(f32::from(EVENT_DOT_PX)),
                height: Val::Px(f32::from(EVENT_DOT_PX)),
                border_radius: BorderRadius::all(Val::Px(f32::from(EVENT_DOT_PX) / 2.0)),
                ..Node::default()
            },
            BackgroundColor(dot_color),
            ChildOf(cell),
        ));
    }
}

fn npc_dot_color(event: &AreaEvent, area_pos: IVec2) -> Option<Color> {
    match event {
        AreaEvent::NpcEncounter(_) => Some(tw::AMBER_400),
        AreaEvent::None if area_pos == IVec2::ZERO => Some(tw::AMBER_400),
        _ => None,
    }
}

fn connector_rect(dir: Direction, cell_left: u16, cell_top: u16) -> Option<(u16, u16, u16, u16)> {
    let half_cell = CELL_PX / 2;
    let lat_offset = (CELL_PX - CONNECTOR_THICKNESS_PX) / 2;
    Some(match dir {
        Direction::North => (
            cell_left + lat_offset,
            cell_top.checked_sub(GAP_PX)?,
            CONNECTOR_THICKNESS_PX,
            GAP_PX,
        ),
        Direction::South => (
            cell_left + lat_offset,
            cell_top + CELL_PX,
            CONNECTOR_THICKNESS_PX,
            GAP_PX,
        ),
        Direction::East => (
            cell_left + CELL_PX,
            cell_top + lat_offset,
            GAP_PX,
            CONNECTOR_THICKNESS_PX,
        ),
        Direction::West => (
            cell_left.checked_sub(GAP_PX)?,
            cell_top + lat_offset,
            GAP_PX,
            CONNECTOR_THICKNESS_PX,
        ),
    })
    .filter(|_| half_cell > 0)
}

fn spawn_legend(
    panel: Entity,
    asset_server: &AssetServer,
    fonts: &UiFont,
    commands: &mut Commands,
) {
    let legend = commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(LEGEND_ROW_GAP_PX),
                ..Node::default()
            },
            ChildOf(panel),
        ))
        .id();

    spawn_legend_row(
        legend,
        commands,
        asset_server,
        fonts,
        LegendIcon::Sprite(PORTAL_ICON_PATH),
        "Portal",
    );
    spawn_legend_row(
        legend,
        commands,
        asset_server,
        fonts,
        LegendIcon::Sprite(ENEMY_ICON_PATH),
        "Enemy",
    );
    spawn_legend_row(
        legend,
        commands,
        asset_server,
        fonts,
        LegendIcon::Dot(tw::AMBER_400),
        "NPC",
    );
}

enum LegendIcon {
    Sprite(&'static str),
    Dot(Color),
}

fn spawn_legend_row(
    legend: Entity,
    commands: &mut Commands,
    asset_server: &AssetServer,
    fonts: &UiFont,
    icon: LegendIcon,
    label: &str,
) {
    let row = commands
        .spawn((
            Node {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: Val::Px(LEGEND_ITEM_GAP_PX),
                ..Node::default()
            },
            ChildOf(legend),
        ))
        .id();

    match icon {
        LegendIcon::Sprite(path) => {
            commands.spawn((
                Node {
                    width: Val::Px(f32::from(LEGEND_DOT_PX) + 1.0),
                    height: Val::Px(f32::from(LEGEND_DOT_PX) + 1.0),
                    ..Node::default()
                },
                ImageNode::new(asset_server.load(path)),
                ChildOf(row),
            ));
        }
        LegendIcon::Dot(color) => {
            commands.spawn((
                Node {
                    width: Val::Px(f32::from(LEGEND_DOT_PX)),
                    height: Val::Px(f32::from(LEGEND_DOT_PX)),
                    border_radius: BorderRadius::all(Val::Px(f32::from(LEGEND_DOT_PX) / 2.0)),
                    ..Node::default()
                },
                BackgroundColor(color),
                ChildOf(row),
            ));
        }
    }
    commands.spawn((
        Text::new(label.to_string()),
        TextFont {
            font: fonts.0.clone(),
            font_size: tw::TEXT_XS_PX,
            ..default()
        },
        TextColor(tw::EMERALD_300),
        ChildOf(row),
    ));
}
