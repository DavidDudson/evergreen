---
name: art-prop
description: Generate world props and scenery for Evergreen locally — barrels, signposts, bushes, trees, wells, crates. Use when asked for a map object, decoration, or anything destined for assets/sprites/scenery.
---

# Prop / Map Object Generation

Props are top-down world objects with transparent backgrounds, sized by
footprint. Local pipeline; no PixelLab credits needed.

**Read `research/art/pixellab_style_guide.md` first** — the grid table and the
map-object description formula are the contract. Append the style suffix
verbatim; do not paraphrase the art direction.

## Sizes (from the grid table)

| Footprint | Sprite px | Examples |
|---|---|---|
| Small, 1 tile | 16 | flowers, mushrooms, pebbles |
| Medium, ~1.5 tiles | 24 | bushes, crates, barrels |
| Large, 2×2 tiles | 32 | trees, wells, signposts |
| Scenery | up to 64 | ruins, large rocks |

## Prompt formula

> [object name], [material], [key visual detail], [colour from palette].
> low top-down view, warm earthy palette, hue-shifted shadows toward cool
> purple, highlights toward warm gold, soft shading, storybook fantasy style,
> centred on a plain background

The camera phrase matters — "low top-down" must appear or the model renders a
side view that will not sit correctly on the tile grid.

## Steps

1. `generate_best(prompt=..., kind="icon", n=4, criteria="<object>, low top-down, single object, clean edges")`
   — `kind="icon"` is right for props too: centred single object with alpha.
2. `pixelize(image_path=<winner>, target_px=<from table>, palette="apollo_forest", upscale=8)`
   — `apollo_forest` drops the hot reds, which keeps woodland props in key.
   Use plain `apollo` for anything deliberately warm (fire, market cloth, gold).
3. Verify the sprite sits inside its footprint with transparent padding — the
   style guide wants canvas ≈ 2× the visual object for large props.
4. Save to `assets/sprites/scenery/`, `snake_case.png`.

## Batching

Ask for a set in one pass, then pixelize each winner:

```
for each of ["wooden barrel with iron bands", "stone well with moss", ...]:
    generate_best(...) → pixelize(...)
```

Local generation has no concurrency limit or credit cost, unlike PixelLab —
render generously and discard.
