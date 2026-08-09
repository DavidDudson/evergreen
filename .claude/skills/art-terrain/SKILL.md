---
name: art-terrain
description: Generate 16x16 terrain tiles for Evergreen — grass, dirt, stone, water, forest floor and their transitions. Explains the seam problem with locally generated tiles and when to use PixelLab tilesets instead. Use for anything destined for assets/sprites/terrain.
---

# Terrain Tile Generation

Read this fully before generating — terrain is the one asset type where the
local rig has a real limitation.

**Style contract:** the terrain section of
`research/art/pixellab_style_guide.md` (`tile_size: 16`, `view: "high top-down"`
— flatter than props — `outline: "selective outline"`).

## The seam problem

Core ComfyUI has no circular-padding node, so **nothing in the local pipeline
generates truly seamless tiles**. A locally made tile will show a visible seam
when repeated unless it is hand-fixed or the texture is near-uniform.

| Need | Use |
|---|---|
| Wang / auto-tile sets, transitions between terrains | **PixelLab** `create_topdown_tileset` (~100 s per set) |
| One-off decorative tile, near-uniform texture (moss, gravel, still water) | **local**, verified with `tile_preview` |
| Texture reference to trace or repaint in Aseprite | **local** |

## Local path

1. Generate the texture larger than the tile, so downsampling averages noise
   into something that repeats acceptably:

   ```
   generate_image(
     prompt="<terrain type>, <surface texture>, <1-2 accent details>, seamless repeating texture, high top-down view, soft natural palette, warm forest greens, rich earthy browns, gentle contrast, storybook forest RPG",
     width=1024, height=1024, backend="zimage"
   )
   ```

2. ```
   pixelize(image_path=..., target_px=16, palette="apollo_forest")
   ```

3. **Always** check the seams before saving:

   ```
   tile_preview(image_path=<16px tile>, grid=3, upscale=8)
   ```

   It returns `seam_diff_horizontal` / `seam_diff_vertical` and
   `wraps_cleanly`. Under ~8 is usually invisible in play; above that, open the
   3×3 preview and look. If it fails, either pick a more uniform texture, or
   hand-fix the wrap in Aseprite (offset by half, repaint the cross seam).

4. Save to `assets/sprites/terrain/`, `snake_case.png`.

## Transitions

Transition tiles (grass overtaking bare earth, shoreline) need to match both
neighbours exactly. Do these in PixelLab as a set, or hand-author them from two
finished tiles — generating each independently will not line up.
