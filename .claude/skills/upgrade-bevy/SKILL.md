---
name: upgrade-bevy
description: Upgrade Bevy across a minor version (e.g. 0.18 -> 0.19). Checks plugin-ecosystem readiness first, scopes the migration guide to code this repo actually uses, then migrates crate-by-crate with build/clippy/test/wasm gates. Use when user says "upgrade bevy", "bump bevy", "migrate to bevy 0.X", "/upgrade-bevy".
tools: Bash, Read, Edit, Write, Grep, Glob, WebFetch
---

# Upgrade Bevy

Bevy minor bumps are not dependency bumps -- they are migrations that touch
every crate in the workspace. This skill is separate from `/upgrade-deps` for
that reason. Run `/upgrade-deps` for everything else; run this only for Bevy.

Five phases: **readiness -> scope -> plan -> migrate -> land**. Each phase has a
gate. Do not skip ahead.

## Phase 0 -- Readiness (read-only, ~2 minutes)

The single most common outcome is "not yet". Determine that before spending
anything else.

### 0.1 What is the latest Bevy?

`cargo info bevy` reads a cached index and goes stale for months. Query the
sparse index directly:

```bash
curl -sA "evergreen-dep-audit (davidjohndudson@gmail.com)" \
  https://index.crates.io/be/vy/bevy | jq -r '.vers' | tail -8
```

Versions containing `-` are pre-releases (`0.19.0-rc.1`). **Never upgrade to a
pre-release** -- plugins do not follow until stable, and rc APIs still churn.

### 0.2 Are the Bevy plugins ready?

This is the gate. Every `bevy_*` plugin pins an exact Bevy minor. If one lags,
the resolver pulls two Bevy versions into the graph and every `Component` /
`Resource` impl mismatches at the type level. There is no workaround short of
forking.

Current plugins in this workspace:

| Plugin | Declared in | What it provides |
| --- | --- | --- |
| `bevy_ecs_tilemap` | `level/Cargo.toml` | tilemap chunk rendering |
| `bevy_light_2d` | `lighting/Cargo.toml` | `Light2d`, `AmbientLight2d`, `PointLight2d`, occluders |

Check each plugin's declared Bevy requirement per published version:

```bash
idx() { c=$1
  case ${#c} in
    1) p="1/$c";; 2) p="2/$c";;
    3) p="3/$(printf %s "$c"|cut -c1)/$c";;
    *) p="$(printf %s "$c"|cut -c1-2)/$(printf %s "$c"|cut -c3-4)/$c";;
  esac
  curl -sA "evergreen-dep-audit (davidjohndudson@gmail.com)" "https://index.crates.io/$p"; }

for c in bevy_ecs_tilemap bevy_light_2d; do
  echo "=== $c"
  idx $c | jq -r 'select(.yanked==false)
    | "\(.vers)  bevy=\([.deps[]|select(.name=="bevy")|.req]|unique|join(","))"' | tail -6
done
```

If a plugin has no release requiring the target Bevy, check whether upstream is
working on one before declaring it blocked:

```bash
R=jgayfer/bevy_light_2d   # or the relevant repo
curl -sA a "https://api.github.com/repos/$R/commits?per_page=8" \
  | jq -r '.[]|"\(.commit.author.date[:10]) \(.commit.message|split("\n")[0])"'
curl -sA a "https://api.github.com/repos/$R/pulls?state=open" \
  | jq -r '.[]|"#\(.number) \(.title) \(.updated_at[:10])"'
curl -sA a "https://raw.githubusercontent.com/$R/main/Cargo.toml" | head -20
```

Judge from the history how long that maintainer usually takes after a Bevy
release. Report the estimate; do not guess silently.

**Gate.** If any plugin is not ready, STOP and report. Present exactly three
options and let the user pick -- do not pick for them:

1. **Wait** (default, almost always right). Bevy stays pinned; `/upgrade-deps`
   still bumps everything else.
2. **Fork the plugin.** Only sane if the plugin is small AND the Bevy release
   did not break the APIs it uses. Weigh it against the render-API churn: a
   plugin with custom render-graph nodes is a rewrite, not a patch. Measure
   first:
   ```bash
   curl -sA a "https://api.github.com/repos/$R/git/trees/main?recursive=1" \
     | jq '[.tree[]|select(.path|test("^src/"))|.size]|add'
   ```
3. **Drop the plugin** and replace its feature with hand-rolled code.

## Phase 1 -- Scope (read-only)

Download the migration guide and cut it down to what this repo actually touches.
Guides run 2000+ lines and are ~90% irrelevant to a 2D wasm game.

```bash
OLD=0.18; NEW=0.19
curl -sA a "https://raw.githubusercontent.com/bevyengine/bevy-website/main/content/learn/migration-guides/$OLD-to-$NEW.md" \
  -o /tmp/mig.md
rg -n '^#{1,3} ' /tmp/mig.md    # index of every breaking change
```

Then grep the workspace for each candidate symbol before reading its section.
A section with zero hits is not a migration task. Useful sweep:

```bash
for s in TextFont TextLayout Text2d Hdr Bloom ComputedNode ExtractComponent \
         RenderLabel FullscreenMaterial DefaultErrorHandler NextState \
         on_replace 'derive\(Resource' 'use bevy::render::'; do
  printf '%-24s %s\n' "$s" "$(rg -c "$s" --type rust | awk -F: '{s+=$2} END{print s+0}')"
done
```

Two classes deserve extra care because they break **quietly** rather than at
compile time:

- **Cargo feature collection changes.** The root `Cargo.toml` opts out of
  default features and lists ~40 features by hand. Bevy reshuffles which
  collection implies which feature nearly every release, so a feature can
  silently stop being enabled and audio/UI/windowing just vanishes at runtime.
  Diff the feature list against the new release's collection tables every time.
- **Render/pipeline semantics** (tonemapping, bloom color space, HDR). These
  compile fine and change how the game looks. Screenshot before and after.

**Gate.** Show the user the scoped list -- "N of M guide sections apply, here
they are" -- before planning.

## Phase 2 -- Plan

Write a checklist ordered by dependency, not by guide order:

1. `Cargo.toml` features and plugin versions
2. `models` (shared types -- everything depends on it)
3. leaf gameplay crates (`combat`, `camera`, `player`, `level`, `dialog`,
   `keybinds`, `save`, `quest`, `lighting`)
4. `post_processing` (render internals -- highest churn, most likely to need a
   real rewrite)
5. `ui` (text and layout APIs churn most releases)
6. `evergreen_main` (plugin wiring)

Each item: what changed, how many call sites, which files. Cross-reference
project memory at
`~/.claude/projects/-home-ddudson-repos-evergreen/memory/` -- previous Bevy
migrations left notes there ([[bevy_18_hdr_marker]], [[bevy_18_sprite_render]],
[[bevy_light_2d_api]], [[bevy_system_tuple_limit]]).

**Gate.** User approves the checklist.

## Phase 3 -- Migrate

```bash
git switch -c chore/bevy-$NEW
```

Bump Bevy in `[workspace.dependencies]` (root `Cargo.toml`) and every
`bevy_*` plugin **in the same commit** -- a half-bumped graph produces
thousands of meaningless type errors.

Then work the checklist top-down. Compile after each crate:

```bash
cargo build -p <crate>          # wasm, the shipping target
```

Do not chase the full error list at once; fix the lowest crate in the
dependency order and re-run. Error counts collapse by an order of magnitude
each time a foundational crate goes green.

If a required API was removed with no replacement, stop and ask. Do not invent
a workaround.

## Phase 4 -- Verification gates

All four must pass. The native and wasm targets diverge -- green on one is not
green on the other.

```bash
# wasm (default target, what ships)
cargo build
cargo clippy

# native (tests only)
nix-shell -p pkg-config wayland libxkbcommon alsa-lib udev libGL vulkan-loader \
  --run "cargo test --target x86_64-unknown-linux-gnu"
```

Notes:
- Tests need `--target x86_64-unknown-linux-gnu`; the workspace defaults to
  wasm. See [[cargo_test_target]].
- The native build links wayland/alsa/udev, which `shell.nix` does not provide
  -- hence the `nix-shell -p` wrapper above.
- `cargo clippy --all-targets` currently fails on `unwrap_used` in test code.
  That is pre-existing; use plain `cargo clippy` as the gate.
- Finally, run `trunk serve` and confirm in the browser: menus render, text is
  laid out, lighting and post-processing look unchanged. Render regressions do
  not fail any of the above.

## Phase 5 -- Land

- Update `.claude/skills/bevy-18/SKILL.md` -> `bevy-19` (rename dir, name,
  description, and every API example inside). Also update its reference in the
  root `CLAUDE.md` skills list.
- Rewrite memory files that name the old version, and add one per API rename
  that cost real debugging time.
- Append to `UPGRADE_NOTES.md`: per-crate before/after, migration guide links,
  files touched, and anything deferred.
- Commit per checklist group, conventional messages
  (`chore(deps): bump bevy 0.18 -> 0.19`, `refactor(ui): migrate TextFont to
  FontSize`).

## Anti-patterns

- Upgrading to a `-rc` release. Plugins never follow; you migrate twice.
- Bumping Bevy without bumping every `bevy_*` plugin in the same commit.
- Reading the migration guide front to back. Grep first, read only what hits.
- Treating a green `cargo build` as done -- feature-collection and color-space
  changes are silent.
- Forking a plugin without first checking whether the release broke the render
  APIs that plugin is built on.
- Skipping the wasm build. This project ships to wasm; native-green is not
  enough.
- Editing `Cargo.lock` by hand.
