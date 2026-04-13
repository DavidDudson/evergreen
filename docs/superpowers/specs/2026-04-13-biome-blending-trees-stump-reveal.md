# Biome Blending, Dense Trees with Stump Reveal, and Decoration Z-Fix

**Date:** 2026-04-13
**Status:** Approved

## Summary

Three interconnected changes to the level rendering system:

1. Biome borders blend 20% into neighboring areas (terrain tiles, scenery, decorations)
2. Trees are regenerated at 48x64 px with biome-specific variants and 1x1 tile trunk colliders
3. A stump-reveal system crossfades tall entities when the player walks behind them
4. Z-ordering is unified so all world entities y-sort correctly

## 1. Biome Border Blending

### Concept

Each area is 32x18 tiles. The outermost 20% of tiles on each side (6 tiles on left/right, 4 tiles on top/bottom) form a **blend zone**. Tiles in the blend zone lerp their effective alignment toward the neighboring area's alignment based on distance from the edge.

### Effective Alignment Calculation

For a tile at position `(x, y)` in an area with alignment `A`:

1. Compute distance from each edge: `dist_left = x`, `dist_right = 31 - x`, `dist_top = 17 - y`, `dist_bottom = y`
2. For each edge within the blend zone (dist < blend_width), check whether a neighbor exists in that direction
3. If a neighbor exists, compute blend factor: `t = 1.0 - (dist / blend_width)` (1.0 at the edge, 0.0 at blend boundary)
4. Lerp: `blended = lerp(area_alignment, neighbor_alignment, t * 0.5)` (max 50% influence at the very edge -- keeps each area's identity)
5. If multiple neighbors influence (corner tiles), take the neighbor with the strongest blend factor

**Blend widths:**
- Horizontal (left/right): 6 tiles (19% of 32)
- Vertical (top/bottom): 4 tiles (22% of 18)

### What Blends

- **Terrain tileset selection**: Tiles in the blend zone use the tileset matching the blended alignment (city/greenwood/darkwood thresholds from `Biome::from_alignment`)
- **Tree density and variant pool**: `tree_threshold()` uses blended alignment. Tree variant selection uses the biome matching the blended alignment.
- **Decoration pool**: Decoration spawning in the blend zone uses the biome matching the blended alignment

### Data Flow

The blend calculation needs neighbor alignments. `spawn_area_tilemap` and `spawn_area_scenery` already receive the `WorldMap` or can access it. The neighbor alignment is looked up via `world.get_area(neighbor_pos).map(|a| a.alignment)`.

If no neighbor exists in a direction (edge of explored world), no blending occurs on that side.

## 2. Tree Overhaul

### New Tree Specs

| Property | Old Value | New Value |
|----------|-----------|-----------|
| Sprite size | 32x32 px | 48x64 px |
| Tile footprint | 2x2 | 3x4 (visual), 1x1 (collider) |
| Collider | 16x16 half-extents | 8x8 half-extents, bottom-center |
| Collider offset | (0, 16) | (0, 4) -- centered on trunk base |
| Anchor | BOTTOM_CENTER | BOTTOM_CENTER |
| Pixel density | 1:1 with 16px grid | 1:1 with 16px grid (48x64 native) |

### Biome Variants

**City (alignment 1-25):**
- `tree_city_ornamental.webp` -- trimmed round canopy, neat appearance
- `tree_city_fruit.webp` -- small fruit tree, civilized

**Greenwood (alignment 26-75):**
- `tree_green_oak.webp` -- broad leafy oak, lush green
- `tree_green_birch.webp` -- slender birch, white bark
- `tree_green_maple.webp` -- full maple, warm green

**Darkwood (alignment 76-100):**
- `tree_dark_gnarled.webp` -- twisted trunk, sparse dark leaves
- `tree_dark_dead.webp` -- bare branches, gray bark
- `tree_dark_willow.webp` -- drooping dark branches, eerie

Each variant also has a corresponding stump sprite:
- `tree_city_ornamental_stump.webp`, etc.
- Stump sprites are the same width (48px) but shorter (~24px tall) -- just the trunk portion

**Total assets:** 8 tree sprites + 8 stump sprites = 16 new sprites

### Density

Tree density is driven by `tree_threshold()` using the (blended) alignment. The current thresholds will be increased to create a dense forest feel:

| Biome | Current base | New base | Edge bonus |
|-------|-------------|----------|------------|
| City | 8 | 5 | +10 (was +20) |
| Greenwood | 30 | 45 | +20 |
| Darkwood | 65 | 80 | +15 |

The smaller 1x1 collider means the player can navigate between tightly packed trunks. Canopies overlap freely above, creating a dense canopy layer.

### `clear_for_tree` Update

The current check prevents trees from overlapping path tiles on a 2x2 footprint. With the new 3x4 visual size but 1x1 collider, the check only needs to verify the trunk tile (1x1) is on grass. Canopy can overhang paths -- that's visually correct for a dense forest.

### Asset Paths

```
assets/sprites/scenery/trees/
  city/
    tree_city_ornamental.webp
    tree_city_ornamental_stump.webp
    tree_city_fruit.webp
    tree_city_fruit_stump.webp
  greenwood/
    tree_green_oak.webp
    tree_green_oak_stump.webp
    tree_green_birch.webp
    tree_green_birch_stump.webp
    tree_green_maple.webp
    tree_green_maple_stump.webp
  darkwood/
    tree_dark_gnarled.webp
    tree_dark_gnarled_stump.webp
    tree_dark_dead.webp
    tree_dark_dead_stump.webp
    tree_dark_willow.webp
    tree_dark_willow_stump.webp
```

### PixelLab Generation

All trees use `create_map_object` with:
- Full tree: `width: 48, height: 64`
- Stump: `width: 48, height: 24`
- `view: "low top-down"`
- `outline: "single color outline"`
- `shading: "basic shading"`
- `detail: "medium detail"`
- Style suffix from `research/art/pixellab_style_guide.md`

Stump prompts should reference the full tree: "Trunk and roots only of [tree description], no canopy, cut-off at waist height, matching the base of the full tree."

## 3. Stump Reveal System

### Concept

When the player moves behind a tall entity (higher world_y, meaning the canopy would visually cover them), the entity crossfades from its full sprite to a "stump" sprite showing just the base.

### Entity Structure

Each revealable entity has two sprite children:
- **Full sprite**: the complete image (tree canopy + trunk, or large decoration)
- **Stump sprite**: the base-only image (trunk, or flattened decoration)

The stump sprite starts with alpha = 0. The full sprite starts with alpha = 1.

### Components

```
Revealable {
    canopy_height_px: f32,  // how far above the base the canopy extends
    reveal_state: RevealState,
}

enum RevealState {
    Full,             // full sprite visible
    Revealing(f32),   // transitioning to stump (progress 0..1)
    Revealed,         // stump visible
    Hiding(f32),      // transitioning back to full (progress 0..1)
}
```

Marker components for the child sprites:
```
FullSprite   // marks the full image child
StumpSprite  // marks the stump image child
```

### Detection

A per-frame system checks all `Revealable` entities against the player position:

- **Trigger condition**: player's y > entity's y AND player's y < entity's y + 16px (1 tile north of base)
- AND player's x is within the entity's visual width / 2
- If triggered and state is `Full` or `Hiding`, transition to `Revealing`
- If not triggered and state is `Revealed` or `Revealing`, transition to `Hiding`

### Transition

- Duration: 0.3 seconds
- `Revealing`: full sprite alpha lerps 1.0 -> 0.0, stump sprite alpha lerps 0.0 -> 1.0
- `Hiding`: reverse
- Uses delta time accumulation, not a Timer (simpler for interruptible transitions)
- If interrupted mid-transition, reverses from current progress

### Which Entities Get Stump Reveal

- **All trees**: yes (48x64 sprites tower over the player)
- **Large decorations (32x16+)**: yes (carts, fallen logs, spider webs)
- **Medium decorations (24x24)**: yes (bushes, crates, barrels)
- **Small decorations (16x16)**: no (too small to cover the player)

For decorations without a dedicated stump sprite, the "stump" behavior is simply fading to alpha = 0.3 (semi-transparent) over the same 0.3s duration rather than swapping to a separate image.

## 4. Unified Y-Sort Z-Ordering

### Problem

Currently each entity type has a fixed z-layer (trees=3, decorations=5, NPCs=9, player=10). This means a decoration that is south of the player (should render in front) always renders behind the player. The y-sort offset (`-world_y * 0.001`) only works within a single layer.

### Solution

All world entities that participate in y-sorting share a single base z value. The z-position is computed purely from y-position:

```
z = WORLD_ENTITY_Z - world_y * Y_SORT_SCALE
```

Where `WORLD_ENTITY_Z` is a single constant (e.g. 5.0) used by trees, decorations, NPCs, and the player. Entities lower on screen (lower y) get a higher z and render in front.

### Layer Changes

| Entity | Old z | New z |
|--------|-------|-------|
| Tilemap | 0 (fixed) | 0 (unchanged) |
| Trees | 3 - y*0.001 | 5 - y*0.001 |
| Decorations | 5 - y*0.001 | 5 - y*0.001 |
| NPCs | 9 (fixed) | 5 - y*0.001 |
| Player | 10 (fixed) | 5 - y*0.001 |
| NPC Labels | 20 (fixed) | 20 (unchanged) |

The `Layer` enum simplifies:
- `Layer::Tilemap` (z=0) -- unchanged
- `Layer::World` (z=5) -- new, replaces SceneryTree + Decoration + Npc + Player
- `Layer::NpcLabel` (z=20) -- unchanged

The player system needs to update its z every frame based on its y-position, same as scenery currently does at spawn time. NPCs need the same treatment.

### Y_SORT_SCALE

The current scale of 0.001 works for the map height. With MAP_H_PX = 288, the z range within one area is 0.288 -- well within the 5.0 base, no risk of overlapping with tilemap (0) or labels (20).

## 5. Mixel Prevention

All sprites must be generated at their exact display resolution:
- Trees: 48x64 native, rendered at 48x64
- Stumps: 48x24 native, rendered at 48x24
- Player: 32x64 native, rendered at 32x64
- Decorations: 16x16, 24x24, 32x16, etc. -- all at native resolution
- No `custom_size` scaling that changes the pixel ratio
- PixelLab canvas size should match or exceed the target sprite size

The 16px tile grid establishes the base pixel density. All sprites align to this: 48 = 3 tiles, 64 = 4 tiles, 24 = 1.5 tiles. No fractional pixel positions at render time.

## Scope

### In Scope
- Biome blend zone calculation and application to terrain/scenery/decorations
- 16 new tree sprites (8 full + 8 stump) via PixelLab
- Delete old tree sprites (tree_oak.webp, tree_pine.webp)
- Stump reveal system with crossfade for trees and large decorations
- Unified y-sort z-ordering for all world entities
- Updated tree density thresholds for dense forest feel
- Updated tree collider (1x1 trunk only)

### Out of Scope
- New decoration sprites (already generated)
- Terrain tileset changes (already done)
- Decoration stump sprites (use alpha fade instead)
- Camera or UI changes
