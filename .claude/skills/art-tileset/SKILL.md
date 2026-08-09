---
name: art-tileset
description: Build dual-grid (corner wang) terrain tilesets for Evergreen from two seamless base tiles -- grass/dirt, grass/water, path/stone. Produces a 16-tile atlas in canonical wang order matching level/src/terrain.rs. Use when asked for a tileset, terrain transitions, autotiling, or a new terrain pair.
---

# Dual-Grid Tileset Generation

Evergreen already renders on a dual grid: `level/src/terrain.rs::wang_index`
takes the four world cells at a drawn tile's corners and packs them
`NW=8, NE=4, SW=2, SE=1`. That means a terrain pair needs **16 tiles**, not the
47 a blob set would.

Transitions are **composited, not generated** -- `dual_grid_set` masks terrain A
over terrain B using a corner falloff field, so adjacent tiles agree along
every shared edge by construction. No AI is involved in the transition step and
none is wanted: generated transitions do not line up.

## Steps

1. **Make two seamless base tiles.** For each terrain, follow `art-terrain`:
   generate a 1024px texture, `make_tileable` it, `pixelize` to 16px, and check
   with `tile_preview` before continuing. Both tiles must be the same size and
   square.

2. **Build the set.**

   ```
   dual_grid_set(
     tile_a="<the 'set' terrain, e.g. grass16.png>",
     tile_b="<the 'unset' terrain, e.g. dirt16.png>",
     output_dir="assets/sprites/terrain",
     name="grass_dirt",
     dither=true,
     palette="apollo_forest"
   )
   ```

   `tile_a` is the terrain whose bit is 1. In `terrain.rs` grass is the set bit
   ("grass wins on a 2-vs-2 tie"), so grass is always `tile_a` for grass pairs.

3. **Wire it up.** The atlas is a 4-column sheet where **atlas index == wang
   index**, so for a set generated this way:

   ```rust
   pub const WANG_TO_ATLAS: [u32; 16] = [0,1,2,3,4,5,6,7,8,9,10,11,12,13,14,15];
   ```

   The current value in `terrain.rs` is a scrambled permutation carried over
   from PixelLab's sheet ordering. Update it (or drop the indirection) when the
   first locally generated set replaces the PixelLab one -- the emitted
   `<name>_atlas.json` states `wang_to_atlas` explicitly, so check it rather
   than assuming.

4. **Eyeball the atlas.** Index 0 must be pure terrain B, index 15 pure terrain
   A, and the four single-corner tiles must show a rounded quarter of A. If
   anything looks inverted, `tile_a`/`tile_b` are the wrong way round.

## Saving (WebP, always)

This repo bans PNG and JPEG in `assets/` -- see CLAUDE.md. Convert before
saving, losslessly, because lossy WebP resamples across hard colour edges and
puts colours back in the file that the palette conform removed:

```
to_webp(image_path=<final sprite>, lossless=true, keep_source=false,
        output_path="assets/sprites/<dir>/<snake_case>.webp")
```

## Options

- `dither=false` for a hard edge -- right for stone/path against dirt, where a
  crisp boundary reads as a built edge rather than a natural one.
- `palette=""` to skip conforming, e.g. when the bases are already conformed
  and you do not want a second quantisation pass.

## Files produced

```
<name>_atlas.png     4x4 sheet, atlas index == wang index
<name>_00.png .. _15.png   individual tiles
<name>_atlas.json    tile size, bit order, wang_to_atlas mapping
```
