---
name: art-icon
description: Generate item, ability and status-effect icons for Evergreen locally (ComfyUI via the gameart MCP), palette-locked to Apollo. Use when asked for an inventory icon, item sprite, ability icon, or anything destined for assets/sprites/icons or assets/sprites/items.
---

# Icon Generation

Local pipeline, no PixelLab credits. Icons are the easiest asset type to get
right locally: single object, centred, no tiling, no rotations.

**Style rules live in `research/art/adamcyounis_style.md` and
`research/art/asset_style_guide.md`.** Read the palette and outline sections
before writing a prompt. Do not restate the style in your own words -- use the
preset fragment verbatim.

## Pick loop

Follow `research/art/local_workflow.md`: five variants in one call →
`contact_sheet` → user picks by number → `describe_style` on the winner →
merge into this asset type's row in `research/art/local_style_presets.md`.
Take the style fragment for this type from that presets file rather than
writing style wording inline.

## Style suffix (seeded default)

This is the seeded `icon` row in `local_style_presets.md`. Once a pick cycle
has refined that row, use the row, not this copy.

> warm earthy palette, hue-shifted shadows toward cool purple, highlights
> toward warm gold, moderate saturation, clean readable silhouette, storybook
> fantasy RPG, 16-bit pixel art aesthetic

## Steps

1. **Render candidates.** Four at once, then let the local VLM pick:

   ```
   generate_best(
     prompt="<subject>, <style suffix>",
     kind="icon", n=5,
     criteria="single <subject>, centred, readable as a 32px icon, clean edges"
   )
   ```

   `generate_icon` directly (backend `"sdxl"`, `transparent=True`) if you want a
   specific seed or the 3d-icon LoRA look. Backend `"zimage"` follows complex
   prompts better but ignores LoRAs.

2. **Pixelize and lock the palette.** Icons are 16 or 32 px per the grid table:

   ```
   pixelize(image_path=<winner>, target_px=32, palette="apollo", upscale=8)
   ```

   The `upscale` copy is for eyeballing only -- ship the 32 px file.

3. **Check it reads small.** Open the un-upscaled sprite. If the silhouette is
   ambiguous at 1:1, regenerate with a simpler subject description rather than
   trying to fix detail at 32 px.

4. **Save** to `assets/sprites/icons/` (UI/ability) or `assets/sprites/items/`
   (world items), named `snake_case.webp`.

## Saving (WebP, always)

This repo bans PNG and JPEG in `assets/` -- see CLAUDE.md. Convert before
saving, losslessly, because lossy WebP resamples across hard colour edges and
puts colours back in the file that the palette conform removed:

```
to_webp(image_path=<final sprite>, lossless=true, keep_source=false,
        output_path="assets/sprites/<dir>/<snake_case>.webp")
```

## Notes

- Alpha is real (BiRefNet matting), not a keyed background -- drop straight in.
- If the cut is ragged around thin features (chains, wisps), rerun `cutout` on
  the 1024 px render before pixelizing; matting at full resolution is cleaner.
- Palette conform happens *after* downscaling on purpose: averaging first, then
  snapping, gives smoother ramps than snapping a 1024 px image and shrinking it.
