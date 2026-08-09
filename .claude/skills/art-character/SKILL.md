---
name: art-character
description: Generate NPC and player character sprites for Evergreen. Covers the local concept/single-view path and when to spend PixelLab credits instead for 4/8-direction rotation sets. Use for anything destined for assets/sprites/npc, player, creatures or enemies.
---

# Character Sprite Generation

Characters are the one asset type where the local rig does **not** replace
PixelLab. Read this before starting, because picking the wrong path wastes
either an hour or a pile of credits.

**Style contract:** `research/art/pixellab_style_guide.md` (character defaults,
description formula) and `research/art/adamcyounis_style.md` (proportions,
outlines, shading).

## Which path

| Need | Use | Why |
|---|---|---|
| Concept art, portrait, promo | **local** | Free, fast, high resolution |
| Single-facing static NPC | **local** | One view is all the sprite needs |
| 4/8-direction walk sets | **PixelLab** | `create_character` produces consistent rotations; diffusion cannot hold a character across angles |
| Animation frames | **PixelLab** | Same reason — frame-to-frame identity |

Diffusion re-invents details every seed. A "walking left" render will not match
the "walking right" one, and no prompt fixes that. Do not attempt rotation sets
locally.

## Local path

Description formula from the style guide:

> [body type] [species/race] [gender presentation] with [hair colour/style],
> wearing [clothing in 1-2 palette colours], [1 distinguishing accessory].
> Warm earthy palette, hue-shifted shadows toward cool purple, highlights
> toward warm gold. Clean readable silhouette, storybook fantasy style.

1. Check `research/characters/` for the character's existing description
   (cadwallader.md, morgana.md, npc_list.md, player_characters.md) and use it
   rather than inventing appearance details.
2. ```
   generate_best(
     prompt="<formula>, chibi proportions, low top-down view, full body, centred",
     kind="sprite", n=4,
     criteria="<character>, readable silhouette at 32px, chibi proportions, single figure"
   )
   ```
   `kind="sprite"` routes to SDXL + the pixel-art LoRA with a real alpha cut.
3. `pixelize(image_path=<winner>, target_px=32, palette="apollo", upscale=8)`
   — 32 for NPCs, 48 for player scale.
4. Save to `assets/sprites/npc/`, `player/`, `creatures/` or `enemies/`.

## PixelLab path

Use the `pixellab` MCP with the character defaults from the style guide —
`size: 32`, `view: "low top-down"`, `body_type: "humanoid"`,
`outline: "single color outline"`, `shading: "basic shading"`,
`proportions: chibi`, `n_directions: 8`, `ai_freedom: 600`.

Budget 3–5 minutes per character; cap at 4–5 concurrent jobs or the API
returns 429.

Afterwards, still run `conform_palette(path, palette="apollo")` on the result —
it costs nothing and guarantees the PixelLab output sits in the same palette as
the locally generated props around it.
