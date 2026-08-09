# Dependency Upgrade -- 2026-08-09

Branch: `chore/upgrade-deps-20260809`

Commits:

- `f359f42 fix(tests): compare ambient colors in linear space; ignore pseudo doctest`
- `d58c363 chore(deps): bump direct deps + refresh lockfile, clearing 2 advisories`

## Summary

- 8 direct deps bumped (7 compatible, 1 major dev-dep).
- Lockfile refresh moved ~90 transitive crates and **cleared both RUSTSEC
  vulnerabilities** (2x 7.5 high in `quick-xml`), plus three unsound/yanked
  warnings.
- 0 source changes required by the upgrade. Two pre-existing test failures
  fixed so the verification gate could run at all.
- **Bevy held at 0.18.1.** Bevy 0.19.0 shipped 2026-06-18 but `bevy_light_2d`
  has no 0.19-compatible release. See [Bevy 0.19 feasibility](#bevy-019-feasibility)
  below.

## Per-crate

### `serde` 1.0.228 -> 1.0.229

- **Type:** compatible (used in `dialog`, `keybinds`, `level`, `models`, `quest`, `save`)
- **Release notes:** https://github.com/serde-rs/serde/releases/tag/v1.0.229
- **Migration:** none

### `serde_json` 1.0.149 -> 1.0.151

- **Type:** compatible (used in `save`)
- **Release notes:** https://github.com/serde-rs/json/releases
- **Migration:** none

### `ron` 0.12.1 -> 0.12.2

- **Type:** compatible (used in `dialog`, `quest` for `.dialog.ron` assets)
- **Release notes:** https://github.com/ron-rs/ron/releases/tag/v0.12.2
- **Migration:** none

### `rand` 0.10.1 -> 0.10.2

- **Type:** compatible (used in `dialog`, `level`)
- **Release notes:** https://github.com/rust-random/rand/releases
- **Migration:** none

### `getrandom` 0.4.2 -> 0.4.3

- **Type:** compatible (used in `dialog`, `level`, both with the `wasm_js` feature)
- **Release notes:** https://github.com/rust-random/getrandom/blob/master/CHANGELOG.md
- **Migration:** none

### `web-sys` 0.3.98 -> 0.3.104

- **Type:** compatible (used in `save`, wasm-only, `Window` + `Storage` features)
- **Migration:** none. Ships in lockstep with `wasm-bindgen`.

### `wasm-bindgen` 0.2.121 -> 0.2.127

- **Type:** compatible (used in `save`, wasm-only)
- **Release notes:** https://github.com/wasm-bindgen/wasm-bindgen/releases
- **Migration:** none
- **Notes:** dropped the `wit-bindgen` component-model codegen path, removing 11
  build-time crates from the graph (`wit-bindgen*`, `wit-component`,
  `wit-parser`, `wasm-encoder`, `wasm-metadata`, `wasmparser`, `wasip3`,
  `id-arena`, `leb128fmt`, `prettyplease`, `anyhow`). Cleared RUSTSEC-2026-0190
  as a side effect.

### `husky-rs` 0.3.3 -> 0.4.0 (dev-dep, MAJOR)

- **Type:** breaking (used by `evergreen_main` to install the git pre-commit hook)
- **Release notes:** https://github.com/pplmx/husky-rs/releases/tag/v0.4.0
- **Migration:** none for this repo.
- **Why it is safe here:** every breaking change in 0.4.0 is scoped to "prek
  mode", which activates only when a `prek.toml`, `.pre-commit-config.yaml`, or
  `.pre-commit-config.yml` exists. This repo has none of those and uses
  standalone `.husky/hooks/pre-commit`, which upstream states "continues to work
  exactly as before". MSRV moved 1.78 -> 1.83; `rust-toolchain.toml` pins
  nightly, so no action.
- **Watch out:** if a prek config is ever added to this repo, 0.4.0 will clear
  `core.hooksPath`, ignore `.husky/` entirely, and hard-fail the build when
  `prek` is not on `PATH`.

## Security

`cargo audit` before: **2 vulnerabilities, 6 warnings**.
`cargo audit` after: **0 vulnerabilities, 2 warnings**.

### Fixed

| Advisory | Crate | Severity | How |
| --- | --- | --- | --- |
| RUSTSEC-2026-0195 | `quick-xml` 0.39.4 | 7.5 high -- unbounded namespace-declaration allocation, memory-exhaustion DoS | lockfile bump to >= 0.41 |
| RUSTSEC-2026-0194 | `quick-xml` 0.39.4 | 7.5 high -- quadratic runtime on duplicate attribute names | lockfile bump to >= 0.41 |
| RUSTSEC-2026-0221 | `event-listener` 5.4.1 | unsound -- `!Send` tags cross thread boundaries via `StackSlot` | -> 5.4.2 |
| RUSTSEC-2026-0186 | `memmap2` 0.9.10 | unsound -- unchecked pointer offset | lockfile bump |
| RUSTSEC-2026-0190 | `anyhow` 1.0.102 | unsound -- `Error::downcast_mut()` | crate dropped from graph |
| (yanked) | `spin` 0.10.0 | yanked release | lockfile bump |

Both `quick-xml` advisories reached us through `wayland-scanner`, i.e. the
Linux-native windowing path only. They were never reachable in the shipped wasm
build, but they were reachable in local native test builds.

### Remaining (both accepted, both transitive through Bevy 0.18)

| Advisory | Crate | Path | Resolution |
| --- | --- | --- | --- |
| RUSTSEC-2024-0436 | `paste` 1.0.15 (unmaintained) | `wgpu-hal` -> `wgpu` -> `bevy_render` | rides Bevy; no action available |
| RUSTSEC-2026-0192 | `ttf-parser` 0.25.1 (unmaintained) | `fontdb` -> `cosmic-text` -> `bevy_text` | **resolved by Bevy 0.19**, which replaces cosmic-text with Parley |

## Pre-existing failures fixed

Neither was caused by the upgrade; both blocked `cargo test` and therefore the
verification gate.

1. `lighting/src/ambient.rs` -- four tests used `assert_eq!(t.color, AMBIENT_*)`.
   `target_for_hour` returns a linearly-interpolated `Color`, so the comparison
   was `Color::LinearRgba` against a `Color::Srgba` palette constant and could
   never pass. Routed through the existing `approx_color` helper, matching the
   dusk/dawn tests alongside them.
2. `keybinds/src/bindings.rs` -- the `Keybinds` rustdoc example is pseudo-code
   (`{ ... }`, no imports) but was fenced as ```` ```rust ````, so rustdoc
   compiled it and failed with 8 errors. Re-fenced as ```` ```ignore ````.

## Verification

| Gate | Command | Result |
| --- | --- | --- |
| wasm build (shipping target) | `cargo build` | pass |
| wasm lint | `cargo clippy` | pass (7 pre-existing style warnings in `level`, `ui`, `dialog`) |
| native tests | `nix-shell -p pkg-config wayland libxkbcommon alsa-lib udev libGL vulkan-loader --run "cargo test --target x86_64-unknown-linux-gnu"` | pass, all suites |
| security | `cargo audit` | 0 vulnerabilities |

Not run: browser smoke test via `trunk serve`. No rendering or gameplay code
changed, so there is nothing for it to catch here.

Environment note: `cargo build --target x86_64-unknown-linux-gnu` fails outside
a nix-shell because `wayland-sys` needs `wayland-client.pc` and `shell.nix` does
not provide it. Pre-existing, unrelated to this upgrade. The `nix-shell -p`
wrapper above is the workaround; consider folding those inputs into `shell.nix`.

## Deferred

### `bevy_ecs_tilemap` 0.18.1 (latest 0.19.0)

Ready and waiting -- 0.19.0 requires `bevy ^0.19.0`. Blocked only by Bevy
itself.

### `bevy_light_2d` 0.9.0 (latest 0.9.0)

Already on latest. There is no 0.19-compatible release. **This is the blocker.**

### `bevy` 0.18.1 (latest 0.19.0)

See below.

---

## Bevy 0.19 feasibility

**Verdict: not possible today. Blocked on `bevy_light_2d`. Recommend waiting.**

### The blocker

| Fact | Detail |
| --- | --- |
| Bevy 0.19.0 released | 2026-06-18 (0.19.0-rc.1 on 2026-05-13) |
| `bevy_ecs_tilemap` | 0.19.0 published, requires `bevy ^0.19.0` -- **ready** |
| `bevy_light_2d` | latest 0.9.0 requires `bevy ^0.18` -- **not ready** |
| `bevy_light_2d` upstream | `main` still declares `bevy = "0.18"`; last commit 2026-03-04 ("Bump for 0.9 release"). No 0.19 branch, PR, or issue. |

Cargo cannot bridge this. Pulling `bevy 0.19` alongside `bevy_light_2d 0.9`
resolves two copies of every Bevy crate, and `Light2d` from `bevy_render 0.18`
is a different type from anything `bevy_app 0.19` will accept. Nothing compiles.

Upstream's historical cadence: Bevy 0.17 -> migrated 2025-10-14; Bevy 0.18 ->
migrated 2026-03-04. That is roughly 4 to 8 weeks after each Bevy release. Bevy
0.19 landed ~7 weeks ago with no visible work started, so the maintainer is
running later than usual this cycle.

### Options if waiting is unacceptable

1. **Swap to `bevy_lit`.** The only viable alternative, and a genuinely close
   API match -- see [Alternative: bevy_lit](#alternative-bevy-lit) below. Has
   one open blocker of its own (WebGL2).
2. **Fork `bevy_light_2d`.** Not recommended. It is ~59 KB of source, but ~60%
   of that is three render-graph nodes (`light_map`, `sdf`, `lighting`) plus
   their pipelines and WGSL. Bevy 0.19 **removed the `RenderGraph` API
   entirely** -- `ViewNode` is gone, render passes are now systems in the
   `Core2d`/`Core3d` schedules. Forking means rewriting exactly the part of the
   crate the release broke hardest, in a subsystem nobody here maintains.
3. **Drop 2D lighting.** `bevy_light_2d` reaches only 6 files
   (`lighting/src/{ambient,torch,exit_light,plugin}.rs`, `camera/src/setup.rs`,
   `models/src/palette.rs`) via four symbols: `Light2dPlugin`, `Light2d`,
   `AmbientLight2d`, `PointLight2d`. Cheap to excise, but the day/night cycle
   and torch lighting are the visual identity of the game.
4. **Wait**, and re-run Phase 0 of `/upgrade-bevy` periodically.

### Alternative: `bevy_lit`

`bevy_lit` (https://github.com/malbernaz/bevy_lit) is the only other maintained
2D lighting crate for Bevy. **`bevy_lit 0.11.0` requires `bevy ^0.19.0`** and
landed 2026-06-22, four days after Bevy 0.19.0 shipped. Its compatibility table
covers every Bevy release from 0.14 onward, so its release cadence is
substantially better than `bevy_light_2d`'s.

Surveyed and rejected: `bevy_magic_light_2d` (last release requires bevy 0.14),
`bevy_incandescent` (bevy 0.13). No other 2D lighting crate was found.

#### API mapping

This workspace uses four symbols across six files, and every one has a direct
counterpart:

| `bevy_light_2d` 0.9 | `bevy_lit` 0.11 | Notes |
| --- | --- | --- |
| `Light2dPlugin` | `Lighting2dPlugin` | `lighting/src/plugin.rs:14` |
| `Light2d { ambient_light: AmbientLight2d { .. } }` on camera | `Lighting2dSettings` on camera; `AmbientLight2d` is a **direct** component (pulled in by `#[require]`) | removes the wrapper indirection in `ambient.rs`; query `&mut AmbientLight2d` instead of `&mut Light2d` |
| `AmbientLight2d { color, brightness }` | `AmbientLight2d { color, intensity }` | field rename only |
| `PointLight2d { color, intensity, radius, falloff, cast_shadows }` | `PointLight2d { color, intensity, inner_radius, outer_radius, falloff, cast_shadows }` | `radius` -> `outer_radius`, add `inner_radius: 0.0` |

`LightOccluder2d` is not used anywhere in this workspace, so the biggest
behavioral difference between the two crates (mesh occluders vs rectangle-only)
does not apply.

Estimated cost: ~30 lines across `lighting/src/{plugin,ambient,torch,exit_light}.rs`
and `camera/src/setup.rs`, plus retuning `TorchConfig`/`ExitLightConfig`
intensities against a different falloff model.

#### The catch: WebGL2

`bevy_lit` **dropped WebGL2 support** (commit `124607d`). This project ships
wasm with the `webgl2` feature, so published `0.11.0` will not run in the
browser as configured.

There is a fix in flight: [PR #26](https://github.com/malbernaz/bevy_lit/pull/26)
readds WebGL2. As of 2026-08-09 it is open, `mergeable_state: clean`, +80/-22
across 13 files, based on `main` (already Bevy 0.19). The contributor reports
all examples passing on both `webgl2` and `x86_64-unknown-linux-gnu`; the
maintainer's last comment (2026-07-14) is "from a first glance this is looking
good". Not merged, and no release contains it.

So the choice becomes:

1. **Wait for PR #26 to merge and ship** -- probably the shortest path to
   Bevy 0.19 overall, and cheaper than waiting on `bevy_light_2d`, whose
   upstream shows no 0.19 activity at all.
2. **Pin `bevy_lit` to the PR branch** (`leomeinel/bevy_lit` branch `webgl`)
   as a git dependency. The diff is small and reviewable, and the base is
   already 0.19. Accepts an unreleased dependency.
3. **Switch the wasm backend to WebGPU** and use published `0.11.0` as-is.
   This is a product decision, not a technical one -- it drops browsers without
   WebGPU. **This is the option currently taken; see below.**

#### Backend switch: WebGL2 -> WebGPU (2026-08-09, temporary)

The root `Cargo.toml` bevy feature list now has `"webgpu"` where it had
`"webgl2"`. The comment above it marks the change as temporary and names the
revert condition (malbernaz/bevy_lit#26 shipping). `webgpu` overrides `webgl2`
in Bevy, so the two are mutually exclusive, not additive.

Follow-on edits:

- `camera/src/setup.rs` -- `Msaa::Off` stays, but its comment no longer claims
  WebGL2 forces it. MSAA is off because pixel art gains nothing from it; the
  "HDR + MSAA crashes on WebGL2" constraint is simply no longer the reason.
- `post_processing/src/atmosphere.rs` -- the 16-byte padding comment now says
  it is a WGSL uniform address space requirement, not a WebGL one, so nobody
  deletes the padding thinking WebGPU made it obsolete.

**Cost:** browsers without WebGPU can no longer run the game at all. There is
no automatic fallback -- `webgpu` replaces the GL backend rather than sitting
in front of it.

**Verified in-browser** (Vivaldi 150 / Chromium, Linux, NVIDIA RTX 4090):

- `navigator.gpu.requestAdapter()` resolves; adapter reports
  `vendor: nvidia, architecture: lovelace`.
- Bevy logs `AdapterInfo { ..., backend: BrowserWebGpu }` -- the backend really
  switched, it did not silently fall back.
- No wgpu, shader, pipeline, or validation errors in the console across the
  whole session.
- Gameplay renders correctly: tilemap, scenery, player, NPC name label, torch
  point light, biome atmosphere darkening, HUD and minimap all present and
  matching a WebGL2 capture of the same scene.
- Informational only: `Some GPU preprocessing are limited on this device`.
  Under WebGL2 the equivalent line is the stronger `GPU preprocessing is not
  supported on this device. Falling back to CPU preprocessing`, plus a warning
  that `OrderIndependentTransparencyPlugin` cannot load for lack of
  `FRAGMENT_WRITABLE_STORAGE`. So WebGPU is the less degraded path of the two.
- Pre-existing and unrelated: `Failed to load asset
  'quests/bigby_sick_animals.quest.ron' ... Expected opening '(' for struct
  'QuestId'`. Present on both backends. Worth fixing separately.

**Not established:** a like-for-like framerate comparison. Attempts to measure
frame rate through the browser-automation bridge returned 1 frame in ~22s on
*both* backends, which means the probe was measuring the extension's injected
context rather than the game loop -- not a real result for either. Two
`Page.captureScreenshot` calls also timed out during the WebGPU run while the
level was still streaming in, but the WebGL2 run was not exercised the same way
at that stage, so this does not isolate to the backend. **Judge perceived
performance by playing it, not from these notes.**

#### Known `bevy_lit` issues worth tracking

- [#24](https://github.com/malbernaz/bevy_lit/issues/24) -- a despawned
  `PointLight2d` can leave its lighting effect on screen. `lighting/src/torch.rs:90`
  removes `PointLight2d` to switch the torch off, which is exactly this code
  path. The one comment on the issue suspects it only triggers when
  `AmbientLight2d` itself is removed (this project resets it rather than
  removing it), but verify the torch toggle before committing to the swap.
- [#25](https://github.com/malbernaz/bevy_lit/issues/25) -- `LightOccluder2d`
  is very expensive even with `cast_shadows` off. Not applicable today; matters
  if occluders are ever adopted.
- `bevy_light_2d` has the mirror-image bug
  ([#62](https://github.com/jgayfer/bevy_light_2d/issues/62): `Visibility::Hidden`
  leaves a stale light), so this failure class is not unique to `bevy_lit`.

### What the migration will cost when it unblocks

Scoped from the [0.18 -> 0.19 migration guide](https://bevy.org/learn/migration-guides/0-18-to-0-19/)
(2342 lines, ~130 breaking changes) against what this workspace actually uses.
Ordered by cost.

#### 1. Text overhaul -- Cosmic Text to Parley (largest mechanical change)

- `TextFont::font` is now a `FontSource`, not `Handle<Font>`. Add `.into()` on
  the handle.
- `TextFont::font_size` is now a `FontSize`, not `f32`. Wrap: `FontSize::Px(24.0)`.
- **Blast radius: 16 files, 65 `font_size:` sites**, concentrated in `ui` (12
  files), plus `level` and `diagnostics`.
- `TextLayout` constructors dropped the `new_with_` prefix:
  `new_with_linebreak` -> `linebreak`, `new_with_justify` -> `justify`,
  `new_with_no_wrap` -> `no_wrap`. One site:
  `level/src/bark_bubbles.rs:58`.
- `TextRoot` / `TextSpanAccess` / `TextSpanComponent` consolidated into one
  `TextSection` trait; `read_span`/`write_span` -> `get_text`/`get_text_mut`.
  No hits in this workspace.
- System font discovery now needs the `bevy/system_font_discovery` feature (and
  `fontconfig` on Linux). This project embeds `default_font` and ships its own
  fonts, so it should not need it -- verify text still renders before assuming.

#### 2. Post-processing -- render graph removed (highest risk)

- `post_processing/src/atmosphere.rs` implements `FullscreenMaterial` with a
  `node_edges()` method. `run_in`/`run_after`/`run_before`/`node_edges` are all
  replaced by a single `schedule_configs(system: ScheduleConfigs<BoxedSystem>)`.
  Rewrite the ordering (`Node2d::Tonemapping` -> `Node2d::EndMainPassPostProcessing`)
  as `.in_set(Core2dSystems::PostProcess).before(tonemapping)`.
- `Core2dSystems::PostProcess` split into `EarlyPostProcess` and `PostProcess`;
  2D also gained a `Prepass`. Pick the right one deliberately.
- `ExtractComponent` refactor: `SyncComponent` is now a subtrait and must be
  implemented to clean up extracted components, or specified via
  `#[extract_component_sync_target(...)]`. Affects `BiomeAtmosphere`.
- `RenderLabel`/`InternedRenderLabel` imports in `atmosphere.rs` largely go
  away with the graph API.
- **This file is small (50 lines) but is the only place in the workspace
  touching render internals. Budget real time for it.**

#### 3. Resources are now Components (sweeping, mostly mechanical)

- `#[derive(Resource)]` now also implements `Component`. A type can no longer
  derive both. **Checked: no type in this workspace derives both. Nothing to
  split.** 48 `Resource`-deriving types across 12 crates all stay as-is.
- `#[reflect(Resource)]` now reflects `Component`; use `ReflectComponent`.
- Broad queries (`Query<EntityMut>`, `Query<Entity>`, `Query<Option<&T>>`) can
  now conflict with `Res<T>` in the same system. Filter with
  `Without<IsResource>`. No broad queries found; watch for new ones.
- `Res<T>`/`ResMut<T>` in generic code needs `R: Resource<Mutability = Mutable>`.
- Non-send resource APIs renamed to "non-send data"
  (`init_non_send_resource` -> `init_non_send`, etc.). No hits.

#### 4. Cargo feature audit (silent breakage -- do this first)

The root `Cargo.toml` disables default features and hand-lists ~40. Three
collection changes matter:

- `audio` is no longer implied by `2d`/`3d`/`ui`. Already listed explicitly. OK.
- `ui` is no longer implied by `2d`/`3d`. Already listed explicitly. OK.
- `bevy_window`, `bevy_input_focus`, `custom_cursor` moved out of `default_app`
  into `common_api` / `ui_api` / `default_platform`. All three are already
  listed explicitly. OK.
- `experimental_bevy_ui_widgets` renamed to `bevy_ui_widgets` and folded into
  `ui`. Not used.
- `bevy_transform`: parallel propagation now needs explicit `multi_threaded`
  rather than riding `std`. Already enabled at the top level. Verify it
  propagates.

Conclusion: the existing feature list appears to survive 0.19 intact, which is
lucky and worth re-verifying rather than trusting.

#### 5. Small renames and one-liners

| Change | Sites |
| --- | --- |
| `Hdr` moved `bevy_render` -> `bevy_camera`; no longer extracted to render world | `camera/src/setup.rs:4` |
| `Replace` lifecycle event -> `Discard`; `on_replace` hook -> `on_discard` | none |
| `DefaultErrorHandler` -> `FallbackErrorHandler` (deprecated alias for one release) | none |
| `ComputedNode::stack_index` -> `ComputedStackIndex` | none (`credits.rs` uses `ComputedNode` but not `stack_index`) |
| `Core` prefix dropped from UI widget components | none |
| `ShaderStorageBuffer` -> `ShaderBuffer` | none |
| `WgpuSettingsPriority::Compatibility` -> `::WebGPU` | none |
| `bevy_scene` renamed to `bevy_world_serialization`; `DynamicSceneBuilder` -> `DynamicWorldBuilder` (now needs `&TypeRegistry`) | feature enabled, no API use |
| `bevy_reflect` root reorganized into `structs`/`enums`/`list`/... modules | import-only, if hit |

#### 6. Behavioral changes that compile clean

- **Bloom luma is now computed in linear space.** `post_processing` tunes bloom
  for a pixel-art look (`bloom_config.rs`); expect it to look different and
  need retuning. Screenshot before and after.
- **`DespawnOnEnter`/`DespawnOnExit` now fire on same-state transitions.** Fixed
  bug, previously they silently did not. Use `NextState::set_if_neq()` to keep
  the old behavior. No `NextState::set()` call sites found, so likely a non-issue
  -- recheck at migration time.
- **Dropping a `Task<T>` on wasm now cancels it** instead of detaching. Call
  `.detach()` to keep the old behavior. `futures-lite` is a direct dep; check
  any spawned task whose handle is dropped.
- `SystemParam` validation now happens at data-fetch time.

#### 7. Free wins from 0.19

- Kills the `ttf-parser` unmaintained advisory (cosmic-text -> Parley).
- Unblocks `bevy_ecs_tilemap` 0.19.0.
- May resolve the `paste` advisory depending on which `wgpu` 0.19 pulls in.

## Follow-ups

- [ ] Re-run `/upgrade-bevy` Phase 0 monthly; the whole gate is "does
      `bevy_light_2d` have a 0.19 release yet".
- [ ] When Bevy 0.19 lands: rename `.claude/skills/bevy-18/` to `bevy-19`,
      update its contents and the `CLAUDE.md` reference.
- [ ] Consider adding `wayland`, `libxkbcommon`, `alsa-lib`, `udev`, `libGL`,
      `vulkan-loader`, `pkg-config` to `shell.nix` `buildInputs` so
      `cargo test --target x86_64-unknown-linux-gnu` works without a
      `nix-shell -p` wrapper.
- [ ] 7 pre-existing clippy style warnings in `level`, `ui`, `dialog` (no-op
      operations, `Some(x).filter`, complex type). Not touched here.
- [ ] `cargo clippy --all-targets` fails on `unwrap_used` in test code. Either
      allow the lint for `#[cfg(test)]` or fix the tests.
