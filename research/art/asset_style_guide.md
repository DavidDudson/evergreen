# Evergreen -- Asset Style Guide

Art direction inspired by AdamCYounis (Apollo palette, Insignia). Every
generated asset should follow the sizes, camera angles and description
conventions below to maintain visual consistency.

## World Context

Evergreen is a top-down 2D RPG set in a magical, never-ending forest. Mortal
civilisation lives in clearings called Glades. The tone is storybook fantasy --
warm, inviting, slightly whimsical -- with an undertone of ancient danger.

## Grid & Size Reference

| Asset Type | Pixel Size | Sprite target | Notes |
|---|---|---|---|
| Terrain tiles | 16x16 | `tile_size: 16` | Base unit. Wang tilesets. |
| Flowers / small props | 16x16 | `width: 32, height: 32` | 1-tile footprint, pad canvas |
| Bushes / medium props | 24x24 | `width: 48, height: 48` | ~1.5 tile footprint |
| Trees / large props | 32x32 | `width: 64, height: 64` | 2x2 tile footprint |
| NPCs (standing) | ~32x32 | `size: 32` | Chibi proportions |
| Player character | 32x64 | `size: 48` (canvas ~68px) | 1 tile wide, 2 tiles tall |
| Large scenery | varies | `width/height` as needed | Keep under 64px wide |

**Camera view**: `"low top-down"` for all assets (matches the existing ~30deg angle).

## Character Defaults

```
description: "<character description -- see prompt formula below>"
size: 32                        # NPCs; 48 for player-scale
view: "low top-down"
body_type: "humanoid"
outline: "single color outline"  # Dark colored outline, not pure black
shading: "basic shading"        # 3-tone: shadow, base, highlight
detail: "medium detail"
proportions: '{"type": "preset", "name": "chibi"}'
n_directions: 8
ai_freedom: 600                 # Moderate -- keep it readable
```

### Character Description Formula

> [body type] [species/race] [gender presentation] with [hair color/style],
> wearing [clothing in 1-2 colors from palette], [1 distinguishing accessory].
> Warm earthy palette, hue-shifted shadows toward cool purple, highlights
> toward warm gold. Clean readable silhouette, storybook fantasy style.

**Examples:**
- "Short stocky old man with long white beard, wearing dark green robes with brown leather belt, carrying a gnarled wooden staff. Warm earthy palette, hue-shifted shadows toward cool purple, highlights toward warm gold. Clean readable silhouette, storybook fantasy style."
- "Young woman with long blonde braided hair and a flower crown, wearing a dark teal dress with cream accents. Warm earthy palette, hue-shifted shadows toward cool purple, highlights toward warm gold. Clean readable silhouette, storybook fantasy style."

## Terrain Tileset Defaults

```
tile_size: {"width": 16, "height": 16}
view: "high top-down"           # Terrain reads best flatter
outline: "selective outline"    # Subtle edges, not heavy borders
shading: "basic shading"
detail: "medium detail"
```

### Terrain Description Formula

> [terrain type], [surface texture], [1-2 accent details]. Soft natural palette,
> warm greens and earthy browns, gentle contrast. Storybook forest RPG.

**Examples:**
- Lower: "dark forest floor, scattered leaves and twigs, damp earth"
  Upper: "lush grass, small wildflowers, soft moss patches"
  Transition: "grass overtaking bare earth with scattered pebbles"

## Tiles Pro Defaults (Props, Decorations)

```
tile_size: 32                   # Or 16 for small items
tile_type: "square_topdown"
tile_view: "low top-down"
outline_mode: "segmentation"    # Cleaner results, no outline artifacts
```

### Tile Description Formula

Number each tile. Keep descriptions short and concrete.

> 1). [object] [material] [color hint] 2). [object] [material] [color hint] ...

**Example:**
- "1). wooden barrel with iron bands 2). stone well with moss 3). market crate with cloth 4). hay bale golden yellow 5). log pile rough bark 6). flower pot with purple blooms"

## Map Object Defaults

```
view: "low top-down"
outline: "single color outline"
shading: "medium shading"
detail: "medium detail"
```

Canvas size should be ~2x the visual footprint to allow transparent padding:
- Small prop (1 tile): `width: 32, height: 32`
- Medium prop (1.5 tiles): `width: 48, height: 48`
- Large prop (2+ tiles): `width: 64, height: 64` or larger

### Map Object Description Formula

> [object name], [material], [key visual detail], [color from palette].
> Warm earthy tones, soft shading, storybook fantasy style.

**Example:**
- "Wooden signpost with hanging plank, weathered brown wood, mossy base. Warm earthy tones, soft shading, storybook fantasy style."

## Color Direction (for all prompts)

These phrases should be included or adapted in every prompt to maintain the
AdamCYounis-inspired palette feel:

| Concept | Prompt Language |
|---|---|
| **Overall palette** | "warm earthy palette" or "soft natural palette" |
| **Shadow hue shift** | "hue-shifted shadows toward cool purple" |
| **Highlight hue shift** | "highlights toward warm gold" or "warm yellow highlights" |
| **Saturation** | "moderate saturation, not oversaturated" |
| **Contrast** | "gentle contrast" (terrain) or "clean readable contrast" (characters) |
| **Mood** | "storybook fantasy style" |
| **Greens** | "warm forest greens" (not neon, not olive) |
| **Browns** | "rich earthy browns" (not muddy, not orange) |
| **Accent colors** | "muted teal", "dusty rose", "aged gold" |

## What to Avoid

- Pure black outlines (`single color black outline`) -- use `single color outline` or `selective outline` instead. AdamCYounis derives outline color from the adjacent interior pixel, one shade darker.
- High detail on small sprites -- muddy at 16-32px
- Neon or oversaturated colors -- breaks the storybook tone
- Flat shading without hue shift -- looks lifeless
- Mixing outline styles across asset types -- pick one and stay consistent
- Side view for top-down assets -- camera angle must match


## Quick Reference Card

Append this style suffix to prompts:

> Warm earthy palette with hue-shifted shadows toward cool purple and highlights
> toward warm gold. Moderate saturation, clean readable forms, storybook fantasy
> RPG style. 16-bit pixel art aesthetic.
