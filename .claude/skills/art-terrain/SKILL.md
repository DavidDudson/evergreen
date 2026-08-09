---
name: art-terrain
description: Generate seamless 16x16 terrain base tiles for Evergreen -- grass, dirt, stone, water, forest floor. Fully local. For transitions between two terrains use art-tileset instead. Use for anything destined for assets/sprites/terrain.
---

# Terrain Base Tile Generation

Produces one seamless base tile per terrain. Transitions are a separate step --
see `art-tileset`, which composites them from two finished base tiles.

**Read `research/art/local_workflow.md` for the pick loop** and take the
`terrain` style fragment from `research/art/local_style_presets.md`. Sizes and
camera come from the terrain section of `asset_style_guide.md`
(`tile_size: 16`, `view: "high top-down"` -- flatter than props).

## Steps

1. **Generate five candidate textures at 1024px.** Terrain is one of the few
   types where you want the render much larger than the tile: downsampling
   averages noise into something that repeats well.

   ```
   generate_image(
     prompt="<terrain type>, <surface texture>, <1-2 accent details>, seamless repeating texture, <terrain style fragment>",
     width=1024, height=1024, variants=5, backend="zimage"
   )
   ```

2. **Contact sheet, user picks, refine the preset** -- the standard loop.

3. **Make it actually seamless.** Diffusion output does not wrap:

   ```
   make_tileable(image_path=<winner>, seed=<seed>)
   ```

   This rolls the texture 50% in both axes -- moving the four edges into a cross
   through the middle -- and inpaints that cross away with SDXL. The result
   wraps as-is.

4. **Pixelize to the grid.**

   ```
   pixelize(image_path=<seamless>, target_px=16, palette="apollo_forest")
   ```

5. **Verify the wrap. Always.**

   ```
   tile_preview(image_path=<16px tile>, grid=3, upscale=8)
   ```

   `seam_diff_*` under ~8 and `wraps_cleanly: true` means it is fine; open the
   3x3 preview anyway. If it fails, rerun `make_tileable` with a different seed
   or pick a more uniform texture -- a busy texture with large features will
   never wrap convincingly at 16px.

6. **Save** to `assets/sprites/terrain/`, `snake_case.webp`.

## Saving (WebP, always)

This repo bans PNG and JPEG in `assets/` -- see CLAUDE.md. Convert before
saving, losslessly, because lossy WebP resamples across hard colour edges and
puts colours back in the file that the palette conform removed:

```
to_webp(image_path=<final sprite>, lossless=true, keep_source=false,
        output_path="assets/sprites/<dir>/<snake_case>.webp")
```

## Notes

- Uniform textures (moss, gravel, grass, still water) survive the 16px
  reduction; anything with large distinct features turns to mush. Describe
  surface, not scenery.
- Keep the base tiles around after saving -- `art-tileset` needs both members of
  a pair as inputs, and regenerating a matching partner later is harder than
  keeping the file.
