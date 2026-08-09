---
name: art-background
description: Generate scene and parallax backgrounds for Evergreen — glade vistas, forest depth layers, sky plates, menu art. Use for wide painted backdrops rather than tiles or sprites.
---

# Background / Parallax Generation

The local rig's strongest asset type: no tiling, no rotation, no small-scale
readability constraint. Z-Image renders these in seconds.

**Style contract:** `research/art/adamcyounis_style.md` for palette;
`research/world/` for what a Glade actually looks like — read the relevant lore
file before inventing scenery.

## Single backdrop

```
generate_best(
  prompt="<scene>, atmospheric, painterly, depth, storybook fantasy, warm forest greens, rich earthy browns, hue-shifted shadows toward cool purple, highlights toward warm gold",
  kind="background", n=4,
  criteria="<scene>, clear depth, no characters, no text, storybook mood"
)
```

Aspects: `"landscape"` 1536×1024, `"wide"` 1536×640 for parallax strips,
`"portrait"` 1024×1536, `"square"`. Pass via `generate_background(aspect=...)`
when you need one specifically.

## Parallax layer sets

Use `generate_layered` — Qwen-Image-Layered returns each element as its own
RGBA plate instead of one flat image, which is exactly what parallax needs:

```
generate_layered(
  prompt="a forest glade: distant misty treeline on the back layer, mid-ground trunks and ferns on the middle layer, foreground bracken silhouette on the front layer",
  layers=3, width=1024, height=1024
)
```

Returns paths back-to-front. Describe what belongs on each layer explicitly —
the model splits on what you name, not on depth it guesses.

## Finishing

1. `refine(image_path=..., scale=2.0, denoise=0.3, prompt="<same scene>")` when
   a background needs more resolution — cheaper and sharper than generating
   large directly.
2. `conform_palette(..., palette="apollo_forest")` to sit in key with the
   sprites. Skip this on menu/promo art that is meant to be richer than
   in-engine assets.
3. Do **not** `pixelize` backgrounds unless they are meant to read as pixel art
   at the same density as the sprites — most parallax plates should stay smooth.
