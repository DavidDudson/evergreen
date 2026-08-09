---
name: art-animation
description: Turn approved Evergreen sprites into animation frames and assemble them into the sprite sheets the engine reads -- character walk/idle/run cycles and looping scenery motion (swaying flora, water, torches, banners). Fully local via Qwen-Image-Edit. Use whenever an asset needs to move, or a sheet needs building or verifying.
---

# Animation

Two halves, one pipeline: **generate the frames**, then **assemble the sheet**.
Assembly is not freehand -- the engine declares an exact grid per asset kind and
a sheet that misses it fails silently as garbled frames, not as an error.

**Read `research/art/local_workflow.md` for the pick loop.** Style fragments
come from `research/art/local_style_presets.md` (`character` for actors,
`scenery-anim` for ambient motion).

Prerequisite: an **approved base sprite** already exists. Animation edits from
that base -- see `art-character` for producing it, `art-prop` for scenery
objects. Never start a cycle from a fresh text-to-image render; identity drifts
frame to frame and the drift is far more visible in motion than at rest.

## Sheet layouts

The grid is set by the Rust side. These are the current values -- if a layout
here disagrees with the code, the code wins and this table is stale.

| Kind | Grid | Cell | Sheet | Declared in |
|---|---|---|---|---|
| NPC | 8 cols (4 idle + 4 walk) x 4 rows (S, E, N, W) | 32x32 | 256x128 | `level/src/npcs.rs`, `level/src/galen.rs` |
| Enemy | 8 x 4, same as NPC | 32x32 | 256x128 | `level/src/enemies.rs` |
| Player | 12 cols (4 idle + 4 walk + 4 run) x 8 rows (S, SW, W, NW, N, NE, E, SE) | 32x64 | 384x512 | `player/src/animation.rs` |
| Scenery loop | 1 row, N cols | your choice | N*W x H | per call site |

Row order for NPCs is `NpcFacing::row()` in `models/src/npc_anim.rs`; the
player's 8-way order is `FacingDirection` in `player/src/animation.rs`.

## Character cycles

1. **One `edit_image` per key pose**, each conditioned on the same approved
   base -- never on the previous frame, or error compounds down the cycle.

   ```
   edit_image(
     image_path="<approved base, 1024px>",
     instruction="same character, mid-stride walking, left leg forward, right arm forward, identical outfit, colours and proportions, same art style",
     seed=<base+n>
   )
   ```

   A 4-frame walk is contact / passing / contact-opposite / passing-opposite.
   Generate the two contacts and let the passing frames be the base pose
   shifted -- diffusion will not give you frame-accurate interpolation, so keys
   are what it is for.

2. **Repeat per direction.** Directions come from the turnaround set
   `art-character` produced, each direction's base driving its own cycle.

3. **Pixelize every frame to the same grid.** Same `target_px` across the whole
   sheet or the character changes size mid-walk:

   ```
   pixelize(image_path=<frame>, target_px=32, palette="apollo")
   ```

   32 for NPCs and enemies. The player is 32x64 -- pixelize to `target_px=64`
   (longest side) and let the builder pad the width.

4. **Assemble.** Frames in row-major order, one per cell, no gaps:

   ```
   uv run scripts/build_sheet.py --layout npc \
     --out assets/sprites/npc/npc_<name>_sheet.webp \
     frames/s_idle_{0..3}.png frames/s_walk_{0..3}.png \
     frames/e_idle_{0..3}.png frames/e_walk_{0..3}.png \
     ...
   ```

   Characters are bottom-anchored by default so feet sit on the same baseline
   in every cell -- a centred anchor makes the sprite bob as its silhouette
   height changes. Short cycles are padded by repeating the last frame; an
   empty cell reads in game as a one-frame flicker.

5. **Verify against the engine grid before committing:**

   ```
   uv run scripts/build_sheet.py --verify --layout npc assets/sprites/npc/npc_<name>_sheet.webp
   ```

## Scenery loops

Frame animation is the primary path for scenery whose **shape** changes --
foliage swaying, a waterfall, fire, a banner, a mill wheel. It is not reserved
for hero props; ambient greenery is exactly what it is for.

1. **Start from the finished static prop** (`assets/sprites/scenery/...`, or a
   fresh `art-prop` pass) and edit it into the extremes of its motion:

   ```
   edit_image(
     image_path="<static prop>",
     instruction="same object, same colours and pixel style, leaves bent to the left by wind, base unchanged",
     seed=<base+n>
   )
   ```

   Keep the anchor still -- whatever touches the ground must not move between
   frames, or the prop appears to slide off its tile.

2. **Prefer `PingPong` over `Loop` for sway.** Two or three frames played out
   and back give a full symmetric cycle, so a sway costs 3 cells rather than 6
   and cannot show the jump a `Loop` makes when its ends do not meet. Set the
   mode on `SpriteAnim` at the spawn site.

3. **Pixelize to the prop's existing grid** -- 16 for small props, 24 bushes,
   32 trees. Matching the static version matters: a swaying fern at a different
   pixel density next to a still one is immediately obvious.

4. **Assemble as a strip:**

   ```
   uv run scripts/build_sheet.py --layout loop --cols 3 --cell 32x32 \
     --anchor bottom \
     --out assets/sprites/scenery/flora_extra/fern_sway.webp \
     frames/fern_{0..2}.png
   ```

5. **Play it.** Spawn with a `TextureAtlasLayout` matching the strip and the
   bundle from `models/src/sprite_anim.rs`:

   ```rust
   let layout = atlas_layouts.add(TextureAtlasLayout::from_grid(
       UVec2::splat(32), 3, 1, None, None,
   ));
   commands.spawn((
       Sprite::from_atlas_image(texture, TextureAtlas { layout, index: 0 }),
       sprite_anim_bundle(
           SpriteAnim::ambient(3).with_mode(SpriteAnimMode::PingPong),
           usize::try_from(tile_hash(pos)).unwrap_or(0) % 3, // de-phase neighbours
       ),
   ));
   ```

   **De-phase identical scenery.** Pass a per-instance start frame derived from
   position, as above. A stand of ferns swaying in lockstep reads as a
   rendering bug, not as wind.

### When the shader accessory is still right

Frame animation is the default, but keep the existing transform/alpha systems
where they genuinely win:

- **Wang-tiled surfaces.** `level/src/water/animation.rs` alpha-pulses all water
  in lockstep precisely because per-tile phase or scale would reveal seams.
  A frame strip on tiled water has the same problem.
- **Wind-reactive motion** that must respond to `models::wind` strength
  continuously -- `level/src/grass.rs` and `scenery::animate_rustle` bend by a
  live value no fixed frame set can cover.
- **Dense instancing**, where a strip's atlas cost outweighs the motion.

Use both together freely: a frame-animated torch can still have its light
pulsed by a shader. Shader-only is the accessory; when in doubt, animate frames.

## Saving (WebP, always)

`build_sheet.py` writes lossless WebP directly and refuses any other extension.
For single frames outside the builder:

```
to_webp(image_path=<final sprite>, lossless=true, keep_source=false,
        output_path="assets/sprites/<dir>/<snake_case>.webp")
```

Lossy WebP resamples across hard colour edges and puts colours back in the file
that the palette conform removed.

## Honest limits

- Diffusion produces **keys, not in-betweens**. Long or snappy cycles (attacks,
  deaths) still need hand work in Aseprite. This gets consistent keys fast.
- Small details flip between edits -- buckle side, hair parting, which hand
  holds the tool. Check every frame against the base before assembling; drift
  invisible in a still is obvious at 8 fps.
- `slice_sheet` is the inverse when a single render already contains several
  poses, or when reworking an existing sheet frame by frame.
