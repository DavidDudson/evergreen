---
name: art-ui
description: Generate UI chrome for Evergreen — dialog frames, panels, buttons, banners, and any art that needs legible text baked in (signage, book pages, labels). Use for anything destined for assets/sprites/ui.
---

# UI Art Generation

Two different tools depending on whether the piece contains readable text.

**Style contract:** `research/art/adamcyounis_style.md` (outline and shading
rules apply to chrome too — no pure black borders).

## Panels, frames, buttons — no text

```
generate_ui_panel(
  prompt="<element>, <ornament description>",
  style="ornate fantasy", transparent=True, variants=4
)
```

Then `conform_palette(path, palette="apollo")`.

Do **not** `pixelize` UI chrome to a small grid unless it is genuinely a pixel
element — dialog frames are usually 9-sliced at higher resolution, and
downsampling destroys the corner detail the slice depends on.

Honest expectation: diffusion is weak at structural, symmetrical chrome.
Treat output as reference for hand-authoring, not ship-ready. Four variants and
picking the least broken corner set is a normal outcome.

## Anything with readable text

Use Ideogram 4 — it is the only local model that renders legible glyphs:

```
generate_text_art(
  prompt="<what the piece looks like>",
  text="<the exact string>",
  aspect="landscape", quality="default"
)
```

- `quality`: `"turbo"` 12 steps for drafts, `"default"` 20, `"quality"` 48 for
  finals.
- Keep `text` short. Long strings still garble — check every glyph before
  saving, and regenerate rather than patching letters by hand.
- Verified working for signage: tavern-sign test rendered its string perfectly.

## Steps

1. Generate (4 variants).
2. `critique(image_paths=[...], criteria="<element>, symmetrical, clean corners, readable at UI scale")`
   to rank them, or just look — for chrome, your eye beats the VLM.
3. `conform_palette(..., palette="apollo")`.
4. Save to `assets/sprites/ui/`.
