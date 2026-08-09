---
name: art-character
description: Generate NPC and player character sprites for Evergreen, including multi-direction turnarounds and animation key poses, entirely locally via Qwen-Image-Edit. Use for anything destined for assets/sprites/npc, player, creatures or enemies.
---

# Character Sprite Generation

Fully local. Identity across directions comes from **editing one approved
sprite**, not from re-prompting -- text-to-image re-invents a character every
seed, but an edit conditioned on the original keeps it.

**Read `research/art/local_workflow.md` for the pick loop**; take the
`character` fragment from `research/art/local_style_presets.md`. Appearance
details come from `research/characters/` -- check `npc_list.md`,
`player_characters.md` or the named character file before inventing anything.

## 1. Base sprite

Description formula from `asset_style_guide.md`:

> [body type] [species/race] [gender presentation] with [hair colour/style],
> wearing [clothing in 1-2 palette colours], [1 distinguishing accessory].

```
generate_sprite(prompt="<formula>, <character style fragment>, front view, full body, centred",
                variants=5, seed=<base>)
```

Contact sheet → user picks → `describe_style` → update the preset. **The
winning sprite is now the identity reference for every other direction.** Save
the 1024px original, not just the downscaled sprite -- edits work from the
full-resolution image.

## 2. Turnarounds

One `edit_image` call per direction, each conditioned on the same base:

```
edit_image(
  image_path="<approved base, 1024px>",
  instruction="show the same character from behind, back view, identical outfit, colours and proportions, same art style",
  lora="Qwen-Edit-2509-Multiple-angles.safetensors",
  lora_strength=1.0
)
```

- The multiple-angles LoRA is trained for exactly this; without it the model
  tends to redraw rather than rotate.
- Directions Evergreen uses: front, back, left, right (4-way) or add the
  diagonals for 8-way.
- Always edit **from the base**, never from the previous direction -- errors
  compound down a chain.
- Check each result against the base for drift: hair colour, accessory
  placement, silhouette height. Regenerate with a different seed rather than
  accepting a near-match; a drifting sprite sheet is obvious in motion.

## 3. Animation

Stop here. Cycles, frame generation and sheet assembly are **`art-animation`**
-- it owns the per-kind grids and the builder script, and the approved base
plus its turnarounds are exactly the input it expects.

## 4. Finish

```
pixelize(image_path=<each direction>, target_px=32, palette="apollo")
```

32 for NPCs, enemies and creatures. The player is **32x64**, not square:
pixelize to `target_px=64` (pixelize works on the longest side) and let the
sheet builder pad the width.

Sheet grids differ per kind and are declared in Rust, not here -- see the
layout table in `art-animation`. In short: NPC and enemy sheets are 8x4 of
32x32 (`level/src/npcs.rs`, `level/src/enemies.rs`), the player is 12x8 of
32x64 (`player/src/animation.rs`).

- `slice_sheet` if a single render already contains multiple poses.
- Save to `assets/sprites/npc/`, `player/`, `creatures/` or `enemies/`.

## Saving (WebP, always)

This repo bans PNG and JPEG in `assets/` -- see CLAUDE.md. Convert before
saving, losslessly, because lossy WebP resamples across hard colour edges and
puts colours back in the file that the palette conform removed:

```
to_webp(image_path=<final sprite>, lossless=true, keep_source=false,
        output_path="assets/sprites/<dir>/<snake_case>.webp")
```

## Honest limits

- Turnarounds are good, not perfect. Small details (buckle side, hair parting)
  can flip. Budget a cleanup pass in Aseprite for hero characters.
- Long animation cycles are still hand work. This gets you consistent keys --
  see `art-animation` for how far the local rig takes a cycle.
