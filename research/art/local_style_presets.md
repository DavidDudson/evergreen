# Local Style Presets

Living file. Each row is the style fragment appended to prompts for one asset
type, refined every time a generated candidate is picked -- see the pick loop in
`local_workflow.md`.

The seeded values below are transcribed from `adamcyounis_style.md` and
`asset_style_guide.md`. They are a starting point, not a result: nothing here
has been through a pick cycle yet.

## Presets

| Asset type | Style fragment | Avoid | Last refined |
|---|---|---|---|
| icon | warm earthy palette, hue-shifted shadows toward cool purple, highlights toward warm gold, moderate saturation, clean readable silhouette, storybook fantasy RPG, 16-bit pixel art aesthetic | pure black outlines, neon saturation, busy detail | seeded |
| prop | low top-down view, warm earthy tones, soft shading, rich earthy browns, warm forest greens, storybook fantasy style, centred with plain background | side view, high detail, harsh contrast | seeded |
| character | chibi proportions, clean readable silhouette, warm earthy palette, hue-shifted shadows toward cool purple, highlights toward warm gold, storybook fantasy style | realistic proportions, pure black outlines, busy costume detail | seeded |
| terrain | high top-down view, soft natural palette, warm forest greens, rich earthy browns, gentle contrast, storybook forest RPG, seamless repeating texture | strong directional shadows, large features, high contrast | seeded |
| ui | ornate fantasy chrome, symmetrical, decorative border, warm earthy palette, aged gold and muted teal accents, clean readable contrast | text, asymmetry, gradients | seeded |
| background | atmospheric, painterly, depth, storybook fantasy, warm forest greens, hue-shifted shadows toward cool purple, highlights toward warm gold | characters, text, UI elements | seeded |
| scenery-anim | low top-down view, warm earthy tones, soft shading, storybook fantasy style, identical object and colours to the reference, motion only in the upper form, base and footprint unchanged | moved anchor, changed silhouette width, new detail between frames, relit | seeded |

## Fixed vocabulary

From the style guide's colour-direction table. Prefer these exact phrases over
synonyms -- they are what the presets converge on.

| Concept | Phrase |
|---|---|
| Palette | "warm earthy palette" / "soft natural palette" |
| Shadows | "hue-shifted shadows toward cool purple" |
| Highlights | "highlights toward warm gold" |
| Saturation | "moderate saturation, not oversaturated" |
| Contrast | "gentle contrast" (terrain) / "clean readable contrast" (characters) |
| Mood | "storybook fantasy style" |
| Greens | "warm forest greens" |
| Browns | "rich earthy browns" |
| Accents | "muted teal", "dusty rose", "aged gold" |

## How to update a row

Replace the fragment with the merge of the old one and `describe_style`'s
`prompt_fragment` from the picked image -- keep agreements, take the winner's
wording where they conflict, stay under ~40 words. Set **Last refined** to the
date and the asset it came from, e.g. `2026-08-09 (mossy shield)`.
