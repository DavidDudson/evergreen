use std::collections::HashSet;

use bevy::math::IVec2;
use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use models::decoration::Biome;
use models::layer::Layer;
use models::palette;
use models::tile::Tile;

use crate::area::{Area, MAP_HEIGHT, MAP_WIDTH};
use crate::biome_registry::BiomeRegistry;
use crate::blending;
use crate::creatures;
use crate::decorations;
use crate::grass;
use crate::npcs;
use crate::scenery;
use crate::shadows::DropShadowAssets;
use crate::terrain::{self, Terrain};
use crate::world::{AreaChanged, WorldMap};

/// Sprout Lands tiles are 16x16 pixels.
pub const TILE_SIZE_PX: u16 = 16;

/// Pixel dimensions of one map area.
#[allow(clippy::as_conversions)]
const MAP_W_PX: f32 = MAP_WIDTH as f32 * TILE_SIZE_PX as f32;
#[allow(clippy::as_conversions)]
const MAP_H_PX: f32 = MAP_HEIGHT as f32 * TILE_SIZE_PX as f32;

/// Convert a tile-based size (width x height in tiles) to a pixel `Vec2`.
pub fn tile_size(width: Tile, height: Tile) -> Vec2 {
    Vec2::new(
        f32::from(width.0) * f32::from(TILE_SIZE_PX),
        f32::from(height.0) * f32::from(TILE_SIZE_PX),
    )
}

/// World-space pixel offset for the centre of an area at `grid_pos`.
pub fn area_world_offset(grid_pos: IVec2) -> Vec2 {
    #[allow(clippy::as_conversions)]
    Vec2::new(grid_pos.x as f32 * MAP_W_PX, grid_pos.y as f32 * MAP_H_PX)
}

// ---------------------------------------------------------------------------
// Components & resources
// ---------------------------------------------------------------------------

/// Marker for any area tilemap entity.
#[derive(Component)]
pub struct AreaTilemap;

/// Marker for individual tile entities (for bulk despawn).
#[derive(Component)]
pub struct AreaTile;

/// Tracks which areas have had their entities spawned.
#[derive(Resource, Default)]
pub struct SpawnedAreas(pub HashSet<IVec2>);

/// Cardinal offsets for the 4 neighbor areas.
const NEIGHBOR_OFFSETS: [IVec2; 4] = [
    IVec2::new(0, 1),  // North
    IVec2::new(0, -1), // South
    IVec2::new(1, 0),  // East
    IVec2::new(-1, 0), // West
];

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

/// Spawn the current area and its neighbors on game start.
#[allow(clippy::too_many_arguments)]
pub fn spawn_initial_areas(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    shadow_assets: Res<DropShadowAssets>,
    wang: Res<crate::wang::WangTilesets>,
    registry: Res<BiomeRegistry>,
    world: Res<WorldMap>,
    mut spawned: ResMut<SpawnedAreas>,
) {
    let current = world.current;
    ensure_area_spawned(
        &mut commands,
        &asset_server,
        &mut atlas_layouts,
        &shadow_assets,
        &wang,
        &registry,
        &world,
        current,
        &mut spawned,
    );
    for offset in &NEIGHBOR_OFFSETS {
        let pos = current + *offset;
        ensure_area_spawned(
            &mut commands,
            &asset_server,
            &mut atlas_layouts,
            &shadow_assets,
            &wang,
            &registry,
            &world,
            pos,
            &mut spawned,
        );
    }
}

/// On area change, spawn any new neighbor areas that haven't been spawned yet.
#[allow(clippy::too_many_arguments)]
pub fn ensure_neighbors_on_area_change(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    shadow_assets: Res<DropShadowAssets>,
    wang: Res<crate::wang::WangTilesets>,
    registry: Res<BiomeRegistry>,
    world: Res<WorldMap>,
    mut spawned: ResMut<SpawnedAreas>,
    mut events: MessageReader<AreaChanged>,
) {
    if events.read().next().is_none() {
        return;
    }
    let current = world.current;
    ensure_area_spawned(
        &mut commands,
        &asset_server,
        &mut atlas_layouts,
        &shadow_assets,
        &wang,
        &registry,
        &world,
        current,
        &mut spawned,
    );
    for offset in &NEIGHBOR_OFFSETS {
        let pos = current + *offset;
        ensure_area_spawned(
            &mut commands,
            &asset_server,
            &mut atlas_layouts,
            &shadow_assets,
            &wang,
            &registry,
            &world,
            pos,
            &mut spawned,
        );
    }
}

/// Despawn all area entities on game exit.
pub fn despawn_all_areas(
    mut commands: Commands,
    tilemaps: Query<Entity, With<AreaTilemap>>,
    tiles: Query<Entity, With<AreaTile>>,
    mut spawned: ResMut<SpawnedAreas>,
) {
    for entity in &tilemaps {
        commands.entity(entity).despawn();
    }
    for entity in &tiles {
        commands.entity(entity).despawn();
    }
    spawned.0.clear();
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

#[allow(clippy::too_many_arguments)]
fn ensure_area_spawned(
    commands: &mut Commands,
    asset_server: &AssetServer,
    atlas_layouts: &mut Assets<TextureAtlasLayout>,
    shadow_assets: &DropShadowAssets,
    wang: &crate::wang::WangTilesets,
    registry: &BiomeRegistry,
    world: &WorldMap,
    area_pos: IVec2,
    spawned: &mut SpawnedAreas,
) {
    if spawned.0.contains(&area_pos) {
        return;
    }
    let in_world = world.get_area(area_pos).is_some();
    let is_off_map_ocean = !in_world && world.has_ocean;
    let dense_forest = Area::dense_forest();
    let area = world.get_area(area_pos).unwrap_or(&dense_forest);
    spawn_area_tilemap(commands, asset_server, registry, world, area, area_pos);
    crate::water::spawn_area_water(commands, asset_server, wang, world, area_pos);
    if is_off_map_ocean {
        // Off-map ocean placeholder -- skip beaches, flora, fauna, scenery,
        // grass, decorations, creatures, NPCs. The full-area ocean tiles
        // already cover the placeholder grass tilemap underneath.
        spawned.0.insert(area_pos);
        return;
    }
    crate::beach::spawn_area_beach(commands, asset_server, wang, world, area_pos);
    crate::water_flora::spawn_area_water_flora(commands, asset_server, world, area_pos);
    crate::water_fauna::spawn_area_water_fauna(commands, asset_server, world, area_pos);
    scenery::spawn_area_scenery_at(
        commands,
        asset_server,
        shadow_assets,
        registry,
        area,
        area_pos,
        world,
    );
    decorations::spawn_area_decorations(commands, asset_server, registry, area, area_pos, world);
    grass::spawn_area_grass(
        commands,
        asset_server,
        shadow_assets,
        registry,
        area,
        area_pos,
        world,
    );
    creatures::spawn_area_creatures(
        commands,
        asset_server,
        shadow_assets,
        registry,
        area,
        area_pos,
        world,
    );
    npcs::spawn_npc_for_area(
        commands,
        asset_server,
        atlas_layouts,
        shadow_assets,
        area,
        area_pos,
    );
    spawn_portal_for_area(commands, asset_server, atlas_layouts, world, area_pos);
    spawned.0.insert(area_pos);
}

const PORTAL_SPRITE_SIZE_PX: f32 = 32.0;
/// Portal sits just above the tilemap floor but below the World layer so
/// NPCs (which Y-sort within `Layer::World`) always render on top.
const PORTAL_Z_BIAS: f32 = 0.85;

fn spawn_portal_for_area(
    commands: &mut Commands,
    asset_server: &AssetServer,
    atlas_layouts: &mut Assets<TextureAtlasLayout>,
    world: &WorldMap,
    area_pos: IVec2,
) {
    let Some(portal) = world.portal else {
        return;
    };
    if portal.area_pos != area_pos {
        return;
    }
    let base = area_world_offset(area_pos);
    let tile_px = f32::from(TILE_SIZE_PX);
    let world_x = base.x - MAP_W_PX / 2.0
        + f32::from(u16::try_from(portal.tile_x).unwrap_or(0)) * tile_px
        + tile_px / 2.0;
    let world_y = base.y - MAP_H_PX / 2.0
        + f32::from(u16::try_from(portal.tile_y).unwrap_or(0)) * tile_px
        + tile_px / 2.0;
    let parent = commands
        .spawn((
            crate::portal::PortalEntity { kind: portal.kind },
            Sprite {
                image: asset_server.load(portal.kind.sprite_path()),
                custom_size: Some(Vec2::splat(PORTAL_SPRITE_SIZE_PX)),
                ..default()
            },
            Transform::from_xyz(world_x, world_y, Layer::Tilemap.z_f32() + PORTAL_Z_BIAS),
        ))
        .id();

    // Mirror portal: spawn Bloody Mary breathing-idle inside the glass as a
    // child of the mirror sprite. Slightly recessed in z so the silver
    // frame paints on top.
    if matches!(portal.kind, crate::portal::PortalKind::Mirror) {
        let layout = atlas_layouts.add(TextureAtlasLayout::from_grid(
            UVec2::splat(MIRROR_MARY_FRAME_PX),
            u32::try_from(crate::portal::MIRROR_MARY_FRAME_COUNT).unwrap_or(4),
            1,
            None,
            None,
        ));
        commands.spawn((
            crate::portal::MirrorMary::default(),
            Sprite {
                image: asset_server.load("sprites/portals/mirror_mary_breathing.webp"),
                texture_atlas: Some(TextureAtlas { layout, index: 0 }),
                custom_size: Some(Vec2::splat(MIRROR_MARY_DISPLAY_PX)),
                ..default()
            },
            Transform::from_xyz(0.0, MIRROR_MARY_OFFSET_Y_PX, 0.01),
            ChildOf(parent),
        ));
    }
}

/// Mirror's Mary animation frame size (square pixels). Each frame is 68x68.
const MIRROR_MARY_FRAME_PX: u32 = 68;
/// Rendered display size for Mary inside the mirror -- slightly smaller than
/// the mirror itself so the silver frame still shows.
const MIRROR_MARY_DISPLAY_PX: f32 = 22.0;
/// Vertical offset within the mirror frame so Mary sits in the glass area
/// (mirror center is the geometric centre, glass is slightly above).
const MIRROR_MARY_OFFSET_Y_PX: f32 = 2.0;

fn spawn_area_tilemap(
    commands: &mut Commands,
    asset_server: &AssetServer,
    registry: &BiomeRegistry,
    world: &WorldMap,
    area: &Area,
    area_pos: IVec2,
) {
    let center_x = u32::from(MAP_WIDTH) / 2;
    let center_y = u32::from(MAP_HEIGHT) / 2;
    let effective_alignment =
        blending::blended_alignment(area.alignment, center_x, center_y, area_pos, world);
    let texture: Handle<Image> = asset_server.load(registry.terrain_tileset(effective_alignment));
    let base = area_world_offset(area_pos);

    let map_size = TilemapSize {
        x: u32::from(MAP_WIDTH),
        y: u32::from(MAP_HEIGHT),
    };
    let tile_size_f32 = f32::from(TILE_SIZE_PX);
    let ts = TilemapTileSize {
        x: tile_size_f32,
        y: tile_size_f32,
    };
    let grid_size: TilemapGridSize = ts.into();

    let tilemap_entity = commands.spawn_empty().id();
    let mut storage = TileStorage::empty(map_size);

    let in_world = world.get_area(area_pos).is_some();

    for x in 0..MAP_WIDTH {
        for y in 0..MAP_HEIGHT {
            let xu = u32::from(x);
            let yu = u32::from(y);
            let tile_pos = TilePos { x: xu, y: yu };
            let idx = if in_world {
                wang_tile_index(xu, yu, area_pos, world)
            } else {
                wang_tile_index_local(xu, yu, area, area_pos, world)
            };
            let tile_color = biome_tile_color(area.alignment, xu, yu, area_pos, world);
            let tile_entity = commands
                .spawn((
                    AreaTile,
                    TileBundle {
                        position: tile_pos,
                        tilemap_id: TilemapId(tilemap_entity),
                        texture_index: TileTextureIndex(idx),
                        color: tile_color,
                        ..Default::default()
                    },
                ))
                .id();
            storage.set(&tile_pos, tile_entity);
        }
    }

    commands.entity(tilemap_entity).insert((
        AreaTilemap,
        TilemapBundle {
            grid_size,
            map_type: TilemapType::Square,
            size: map_size,
            storage,
            texture: TilemapTexture::Single(texture),
            tile_size: ts,
            transform: Transform::from_translation(Vec3::new(
                base.x - (MAP_W_PX / 2.0),
                base.y - (MAP_H_PX / 2.0),
                Layer::Tilemap.z_f32(),
            )),
            ..Default::default()
        },
    ));
}

/// Wang corner tile index for a cell, consulting adjacent areas across boundaries.
fn wang_tile_index(x: u32, y: u32, area_pos: IVec2, world: &WorldMap) -> u32 {
    let lx = i32::try_from(x).expect("x fits i32");
    let ly = i32::try_from(y).expect("y fits i32");

    let at = |dx: i32, dy: i32| world.terrain_at_extended(area_pos, lx + dx, ly + dy);

    let corner = |a, b, c, d: Option<Terrain>| -> bool {
        [a, b, c, d]
            .iter()
            .filter(|t| **t == Some(Terrain::Grass))
            .count()
            >= 2
    };

    let nw = corner(at(0, 0), at(-1, 0), at(0, 1), at(-1, 1));
    let ne = corner(at(0, 0), at(1, 0), at(0, 1), at(1, 1));
    let sw = corner(at(0, 0), at(-1, 0), at(0, -1), at(-1, -1));
    let se = corner(at(0, 0), at(1, 0), at(0, -1), at(1, -1));

    let wang = terrain::wang_index(nw, ne, sw, se);
    #[allow(clippy::as_conversions)]
    terrain::WANG_TO_ATLAS[wang as usize]
}

/// Wang tile index for an area not in the world map (dense forest fallback).
fn wang_tile_index_local(x: u32, y: u32, area: &Area, area_pos: IVec2, world: &WorldMap) -> u32 {
    let lx = i32::try_from(x).expect("x fits i32");
    let ly = i32::try_from(y).expect("y fits i32");

    let at = |dx: i32, dy: i32| -> Option<Terrain> {
        let nx = lx + dx;
        let ny = ly + dy;
        if let (Ok(ux), Ok(uy)) = (u32::try_from(nx), u32::try_from(ny)) {
            if let Some(t) = area.terrain_at(ux, uy) {
                return Some(t);
            }
        }
        world
            .terrain_at_extended(area_pos, nx, ny)
            .or(Some(Terrain::Grass))
    };

    let corner = |a, b, c, d: Option<Terrain>| -> bool {
        [a, b, c, d]
            .iter()
            .filter(|t| **t == Some(Terrain::Grass))
            .count()
            >= 2
    };

    let nw = corner(at(0, 0), at(-1, 0), at(0, 1), at(-1, 1));
    let ne = corner(at(0, 0), at(1, 0), at(0, 1), at(1, 1));
    let sw = corner(at(0, 0), at(-1, 0), at(0, -1), at(-1, -1));
    let se = corner(at(0, 0), at(1, 0), at(0, -1), at(1, -1));

    let wang = terrain::wang_index(nw, ne, sw, se);
    #[allow(clippy::as_conversions)]
    terrain::WANG_TO_ATLAS[wang as usize]
}

/// Base tile tint color for a biome.
fn biome_base_tint(biome: Biome) -> Color {
    match biome {
        Biome::City => palette::BIOME_CITY_TINT,
        Biome::Greenwood => palette::BIOME_GREENWOOD_TINT,
        Biome::Darkwood => palette::BIOME_DARKWOOD_TINT,
    }
}

/// Compute a per-tile tint color that lerps between biome tints at area borders.
/// Tiles in the interior get the area's base biome tint. Tiles in the blend zone
/// shift toward the neighbor's biome tint (Minecraft-style biome color blending).
fn biome_tile_color(
    area_alignment: u8,
    x: u32,
    y: u32,
    area_pos: IVec2,
    world: &WorldMap,
) -> TileColor {
    let blend = blending::blend_at(area_alignment, x, y, area_pos, world);
    let area_tint = biome_base_tint(Biome::from_alignment(area_alignment));

    if blend.factor < 0.01 {
        return TileColor(area_tint);
    }

    let neighbor_align = match blend.neighbor_alignment {
        Some(a) => a,
        None => return TileColor(area_tint),
    };
    let neighbor_tint = biome_base_tint(Biome::from_alignment(neighbor_align));

    // Lerp RGB channels by the blend factor (0.0 = area color, 0.5 = halfway).
    let t = blend.factor;
    let lerp_channel = |a: f32, b: f32| a + (b - a) * t;

    let a = area_tint.to_srgba();
    let b = neighbor_tint.to_srgba();
    #[allow(clippy::disallowed_methods)]
    let blended = Color::srgba(
        lerp_channel(a.red, b.red),
        lerp_channel(a.green, b.green),
        lerp_channel(a.blue, b.blue),
        1.0,
    );
    TileColor(blended)
}
