# Local Art Workflow -- Generate, Pick, Refine

All Evergreen art is generated locally through the `gameart` MCP server
(ComfyUI on `127.0.0.1:8188` + qwen3-vl on Ollama). No cloud services, no
credits, no rate limits. No cloud art service is required for any asset type.

The `art-*` skills each cover one asset type. This file holds the parts they
share: the pick loop, and the presets that loop maintains.

`art-animation` is the exception -- it takes an already-approved sprite and
makes it move, so it runs the pick loop only on the frames it generates, never
on the identity. It also owns sheet assembly (`scripts/build_sheet.py`) for
every asset kind, character and scenery alike.

## The pick loop

Every asset goes through the same five steps. Do not skip step 4 -- it is what
makes the style converge instead of drifting seed to seed.

1. **Load the preset.** Read the row for this asset type in
   `local_style_presets.md` and append its `style fragment` to the prompt. If
   the row is still the seeded default, say so when presenting results.

2. **Generate five variants.** Always five, always one call:

   ```
   generate_icon(prompt=..., variants=5, seed=<base>)     # or the tool for the type
   ```

   One batched call shares a base seed and walks it, so variants differ by seed
   alone and the comparison is honest.

3. **Present them as one sheet.**

   ```
   contact_sheet(image_paths=<paths>, cols=5)
   ```

   Give the user the sheet path and ask which number wins. The returned
   `index_to_path` maps their answer to a file. Offer `critique(...)` as a
   second opinion, but the user's pick decides.

4. **Refine the preset from the winner.**

   ```
   describe_style(image_path=<winner>)
   ```

   Merge the returned `prompt_fragment` into that asset type's row in
   `local_style_presets.md`: keep what the winner and the existing preset agree
   on, replace what they contradict, and note the date. Keep the fragment under
   ~40 words -- a preset that grows without bound stops steering anything.

   If the user rejects all five, do the opposite: record what to avoid in the
   row's **avoid** column and regenerate before touching the main fragment.

5. **Finish and save.** Per-type post-processing (`pixelize`,
   `conform_palette`, `dual_grid_set`) then the path from the skill.

## Why five

Diffusion output varies more between seeds than between prompt tweaks. Five
1024px Z-Image renders cost about 15 seconds total on the 4090, so generating
five and picking is strictly faster than iterating on prompt wording -- and the
picks are what teach the presets.

## Style contract

The palette and rendering rules come from `adamcyounis_style.md`. They are
enforced two ways, and both matter:

- **In the prompt** -- the style fragment steers the render.
- **After the render** -- `conform_palette` / `pixelize` snap the output to
  Apollo exactly. Prompting alone cannot hold a 46-colour palette.

`asset_style_guide.md` remains the reference for sizes, camera angles and
description formulas.
