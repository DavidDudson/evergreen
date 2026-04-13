# AdamCYounis Pixel Art Style Analysis

## Who is AdamCYounis?

Australian solo indie game developer and pixel artist. Founder of **Uppon Hill** studio (est. 2017). Twitch Partner. Possibly the most prolific pixel art YouTuber -- hundreds of free tutorials covering software, workflows, pixel art styles, animation, game development, and pixel art theory. Currently developing **Insignia** (a Metroidvania). All development is done live on Twitch with documentation on YouTube ("Indie Tales" series).

- Website: upponhill.com (open-door studio philosophy -- shares methods/techniques freely)
- Lospec: lospec.com/adamcyounis
- PixelSchool: pixelschool.org (structured pixel art courses with assignments)
- Patreon: patreon.com/adamcyounis
- Itch.io: uppon-hill.itch.io (demos, free parallax background assets)
- Notable creation: **Apollo palette** (46 colors, 182k+ downloads)
- YouTube playlist: youtube.com/playlist?list=PLLdxW--S_0h4dlWUpl-TzBp-ulqK3NiM_

## Core Philosophy

1. **Readability above all** -- if the sprite is readable, it's playable. Clarity of silhouette and form trumps detail.
2. **Storytelling through art** -- tutorials emphasize crafting pixel art that conveys story or emotion, not just technical execution.
3. **Trust your eyes** -- reject rigid mathematical progressions for color ramps. What "looks good" matters more than precision.
4. **Practical game-ready art** -- everything is oriented toward usable game assets, not portfolio showpieces.
5. **Not pixel-perfect** -- Insignia deliberately uses sub-pixel rotation/scaling for smoother animation. Pragmatism over purity.

## The Apollo Palette (46 Colors)

AdamCYounis's signature palette. Tagged: `16bit`, `linear`. Organized into 6 chromatic ramps of 6 colors each + a 10-step grayscale.

### Color Families (dark to light)

| Family | Hex Values |
|---|---|
| Deep Purple/Red | #241527, #411d31, #752438, #a53030, #cf573c, #da863e |
| Warm Red/Orange | #341c27, #602c2c, #884b2b, #be772b, #de9e41, #e8c170 |
| Brown/Skin | #4d2b32, #7a4841, #ad7757, #c09473, #d7b594, #e7d5b3 |
| Cool Blue | #172038, #253a5e, #3c5e8b, #4f8fba, #73bed3, #a4dddb |
| Forest Green | #19332d, #25562e, #468232, #75a743, #a8ca58, #d0da91 |
| Dark Purple/Pink | #1e1d39, #402751, #7a367b, #a23e8c, #c65197, #df84a5 |
| Grayscale | #090a14, #10141f, #151d28, #202e37, #394a50, #577277, #819796, #a8b5b2, #c7cfcc, #ebede9 |

### Ramp Characteristics

- Each ramp has **smooth hue shifting** -- hue and value shift simultaneously
- Darks converge toward cool purples/blues
- Lights converge toward warm yellows/creams
- **Non-linear value steps**: larger jumps between bright colors, smaller jumps between dark colors
- Grayscale has a slight cool-blue tint (not pure neutral gray)

## Palette Construction Method

1. **Set saturation baseline**: Start at 50-60% saturation across the whole palette
2. **Build ramps with dual shifting**: Shift hue AND value simultaneously. Going lighter? Shift hue toward warm/yellow. Going darker? Shift toward cool/purple.
3. **Start with more colors than needed**: Easier to remove excess than to discover gaps later
4. **Boost mid-saturation**: Increase saturation in the upper-middle third of ramps, desaturate the extremes so lights and darks "bleed into each other"
5. **Test on canvas**: Place colors together, remove what doesn't work, refine by eye
6. **Validate coverage**: Use Aseprite's Indexed Color Mode to check palette against a spectrum slice

## Outline Style

- **Selective/colored outlines** -- not pure black. Outline color is derived from the adjacent interior color, shifted one shade darker.
- Sel-out (selective outlining): the solid outline is broken in areas where light hits the sprite, replaced by a lighter color.
- Insignia sprites consistently show colored/selective outlines rather than hard black.
- Interior detail lines lighter than exterior outlines.
- Consistency is key: pick one outline approach and stick with it across all assets.

## Shading Approach

- **3-tone shading** as foundation: shadow, base, highlight
- Highlights typically outline the top half of forms (top-down light source)
- Hue-shifted shadows (not just darker versions of the base -- shift toward complementary/cool)
- Hue-shifted highlights (shift toward warm/yellow)
- This creates depth and vibrancy that flat darkening/lightening cannot achieve

## Sprite Design Principles

- **Silhouette first** -- simple, recognizable shapes that read at small sizes
- Clean forms over busy detail
- Body type and body language readable even as a solid fill
- At small sizes (16-32px), every pixel is a design choice
- Characters should be distinguishable at a glance

## Subpixel Animation Techniques

Adam teaches three core subpixel animation methods:

1. **Smearing** -- leaving parts of a sprite behind, stretching/squashing pixels to stagger movement across frames, creating the illusion of sub-pixel motion speed.
2. **Outline Tweening** -- smoothly transitioning an object through pixels by introducing information from adjacent rows/columns gradually over multiple frames.
3. **Value/Color Tweening** -- blending colors between pixel positions to create smooth, gradual transitions. The most commonly used technique.

## Key Tutorial Series ("Pixel Art Class")

- Art Styles for Indie Games (style analysis and philosophy)
- Palettes & Colour (palette construction in Aseprite)
- Lighting & Shading Basics (light sources, shading forms)
- Top Down Style Analysis & Tutorial (top-down perspective breakdown)
- Isometric Tile Basics / Isometric Character Basics
- Character Sprite Build (character construction)
- Sub-Pixel Animation (subpixel techniques)
- Tips & Scripts for Supercharging Your Aseprite Workflow

## Relevance to Evergreen

AdamCYounis's style aligns well with Evergreen's existing art direction:
- Both favor readability and charm over hyper-detail
- The Apollo palette's warm earth tones and forest greens suit a fantasy woodland setting
- His chibi/cartoon proportions match the existing character sprites
- The 16-bit aesthetic fits the 16px tile / 32px character scale
- His emphasis on hue-shifted shadows would enhance the existing storybook feel

## Sources

- upponhill.com
- lospec.com/adamcyounis
- lospec.com/palette-list/apollo
- pixelschool.org
- patreon.com/adamcyounis
- uppon-hill.itch.io
- insigniagame.tumblr.com
- AdamCYounis YouTube channel -- Pixel Art Class playlist
- stevelilleyschool.blogspot.com/2021/02/designing-pixel-art-color-palettes-in.html (palette tutorial notes)
- jellytempo.com/pixel-art-youtubers-for-beginners-to-subscribe-to/
- hiitselsie.medium.com/learn-pixel-art-with-free-youtube-videos (YouTuber analysis)
- beacons.ai/i/blog/pixel-art-youtubers (top 10 pixel art YouTubers)
- x.com/AdamCYounis/status/1880961301301821653 (Insignia not pixel-perfect)
- toolify.ai/ai-news/master-the-art-of-subpixel-animation-in-pixel-art-class-204814 (subpixel summary)
