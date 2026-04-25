use bevy::math::IVec2;
use bevy::prelude::*;
use level::area::{AreaEvent, Direction};
use level::world::WorldMap;
use models::palette;

// ---------------------------------------------------------------------------
// Layout constants
// ---------------------------------------------------------------------------

/// Width of one area cell in the minimap (px).
const CELL_W_PX: u16 = 20;
/// Height of one area cell (px).
const CELL_H_PX: u16 = 13;
/// Gap between cells; exit connectors are drawn here.
const GAP_PX: u16 = 4;
/// Horizontal step from one cell origin to the next.
const STEP_W_PX: u16 = CELL_W_PX + GAP_PX;
/// Vertical step from one cell origin to the next.
const STEP_H_PX: u16 = CELL_H_PX + GAP_PX;

/// How many areas to show in each direction from the current one.
const VIEW_RADIUS: i32 = 2;
/// Total diameter of the view (5x5 grid = VIEW_RADIUS * 2 + 1).
const VIEW_SIZE: u16 = 5;

/// Inner padding around the cell grid inside the container.
const PADDING_PX: u16 = 4;

const CONTAINER_W_PX: u16 = VIEW_SIZE * STEP_W_PX - GAP_PX + PADDING_PX * 2;
const CONTAINER_H_PX: u16 = VIEW_SIZE * STEP_H_PX - GAP_PX + PADDING_PX * 2;

/// Distance from the top of the screen (leaves room for HUD health text).
const MINIMAP_TOP_PX: u16 = 24;
/// Distance from the right edge of the screen.
const MINIMAP_RIGHT_PX: u16 = 5;

/// Size of the event icon dot inside a cell.
const EVENT_DOT_SIZE_PX: u16 = 5;
/// Size of the icon-sprite events (enemy, portal). Slightly larger so the
/// pixel-art icon is legible.
const EVENT_ICON_SIZE_PX: u16 = 9;
/// Asset paths for sprite-based event icons.
const PORTAL_ICON_PATH: &str = "sprites/icons/portal.webp";
const ENEMY_ICON_PATH: &str = "sprites/icons/enemy.webp";

// ---------------------------------------------------------------------------
// Components
// ---------------------------------------------------------------------------

#[derive(Component)]
pub struct MinimapRoot;

/// Marker for area cells and connectors so they can be bulk-despawned.
#[derive(Component)]
pub struct MinimapCell;

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

/// Spawn the minimap container and the initial cells for the starting area.
pub fn setup(mut commands: Commands, asset_server: Res<AssetServer>, world: Res<WorldMap>) {
    let root = commands
        .spawn((
            MinimapRoot,
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(f32::from(MINIMAP_TOP_PX)),
                right: Val::Px(f32::from(MINIMAP_RIGHT_PX)),
                width: Val::Px(f32::from(CONTAINER_W_PX)),
                height: Val::Px(f32::from(CONTAINER_H_PX)),
                overflow: Overflow::clip(),
                ..Node::default()
            },
            BackgroundColor(palette::MINIMAP_BG),
        ))
        .id();

    build_cells(root, &asset_server, &world, &mut commands);
}

/// Despawn all minimap elements when leaving the Playing state.
pub fn despawn(mut commands: Commands, root_q: Query<Entity, With<MinimapRoot>>) {
    for entity in &root_q {
        commands.entity(entity).despawn();
    }
}

/// Rebuild minimap cells whenever the world changes (area transition or
/// re-entering Playing from Dialogue/Pause).
pub fn refresh(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    world: Res<WorldMap>,
    root_q: Query<Entity, With<MinimapRoot>>,
    cell_q: Query<Entity, With<MinimapCell>>,
) {
    if !world.is_changed() {
        return;
    }

    let Ok(root) = root_q.single() else { return };

    for entity in &cell_q {
        commands.entity(entity).despawn();
    }

    build_cells(root, &asset_server, &world, &mut commands);
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn build_cells(
    root: Entity,
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

            let color = if area_pos == current {
                palette::MINIMAP_CURRENT
            } else {
                palette::MINIMAP_ROOM
            };

            // Grid position within the minimap view (0 = top-left corner).
            let col = u16::try_from(dx + VIEW_RADIUS).unwrap_or(0);
            let row = u16::try_from(dy + VIEW_RADIUS).unwrap_or(0);
            let cell_left = col * STEP_W_PX + PADDING_PX;
            let cell_top = row * STEP_H_PX + PADDING_PX;

            // Area cell rectangle.
            let cell_entity = commands
                .spawn((
                    MinimapCell,
                    Node {
                        position_type: PositionType::Absolute,
                        left: Val::Px(f32::from(cell_left)),
                        top: Val::Px(f32::from(cell_top)),
                        width: Val::Px(f32::from(CELL_W_PX)),
                        height: Val::Px(f32::from(CELL_H_PX)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..Node::default()
                    },
                    BackgroundColor(color),
                    ChildOf(root),
                ))
                .id();

            // Portal area trumps any other event marker -- spawn the
            // portal icon sprite. Otherwise fall back to enemy icon or
            // colored dot for NPC encounters.
            let is_portal_area = world
                .portal
                .as_ref()
                .is_some_and(|p| p.area_pos == area_pos);
            if is_portal_area {
                commands.spawn((
                    MinimapCell,
                    Node {
                        width: Val::Px(f32::from(EVENT_ICON_SIZE_PX)),
                        height: Val::Px(f32::from(EVENT_ICON_SIZE_PX)),
                        ..Node::default()
                    },
                    ImageNode::new(asset_server.load(PORTAL_ICON_PATH)),
                    ChildOf(cell_entity),
                ));
            } else if matches!(area.event, AreaEvent::Enemy { .. }) {
                commands.spawn((
                    MinimapCell,
                    Node {
                        width: Val::Px(f32::from(EVENT_ICON_SIZE_PX)),
                        height: Val::Px(f32::from(EVENT_ICON_SIZE_PX)),
                        ..Node::default()
                    },
                    ImageNode::new(asset_server.load(ENEMY_ICON_PATH)),
                    ChildOf(cell_entity),
                ));
            } else if let Some(dot_color) = event_dot_color(&area.event, area_pos) {
                commands.spawn((
                    MinimapCell,
                    Node {
                        width: Val::Px(f32::from(EVENT_DOT_SIZE_PX)),
                        height: Val::Px(f32::from(EVENT_DOT_SIZE_PX)),
                        border_radius: BorderRadius::all(Val::Px(
                            f32::from(EVENT_DOT_SIZE_PX) / 2.0,
                        )),
                        ..Node::default()
                    },
                    BackgroundColor(dot_color),
                    ChildOf(cell_entity),
                ));
            }

            // Exit connectors -- small rectangles bridging the gap to the
            // next cell in each exit direction.
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
                            ..Node::default()
                        },
                        BackgroundColor(color),
                        ChildOf(root),
                    ));
                }
            }
        }
    }
}

/// Returns the dot color for an area event, or `None` for no icon.
/// Galen at origin also gets a dot.
fn event_dot_color(event: &AreaEvent, area_pos: IVec2) -> Option<Color> {
    match event {
        AreaEvent::NpcEncounter(_) => Some(palette::MINIMAP_NPC),
        AreaEvent::Enemy { .. } => Some(palette::MINIMAP_ENEMY),
        AreaEvent::None if area_pos == IVec2::ZERO => Some(palette::MINIMAP_NPC),
        AreaEvent::None => Option::None,
    }
}

/// Return `(left, top, width, height)` for the connector rectangle that
/// bridges the GAP between two adjacent cells in the given direction.
/// Returns `None` when the connector would extend outside the container.
fn connector_rect(dir: Direction, cell_left: u16, cell_top: u16) -> Option<(u16, u16, u16, u16)> {
    let con_w = CELL_W_PX / 2;
    let con_h = CELL_H_PX / 2;

    Some(match dir {
        Direction::North => (
            cell_left + CELL_W_PX / 4,
            cell_top.checked_sub(GAP_PX)?,
            con_w,
            GAP_PX,
        ),
        Direction::South => (
            cell_left + CELL_W_PX / 4,
            cell_top + CELL_H_PX,
            con_w,
            GAP_PX,
        ),
        Direction::East => (
            cell_left + CELL_W_PX,
            cell_top + CELL_H_PX / 4,
            GAP_PX,
            con_h,
        ),
        Direction::West => (
            cell_left.checked_sub(GAP_PX)?,
            cell_top + CELL_H_PX / 4,
            GAP_PX,
            con_h,
        ),
    })
}
