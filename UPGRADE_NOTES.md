# Dependency Upgrade -- 2026-05-16

Branch: `chore/upgrade-deps-20260516`
Commit: `e2edf7c chore(deps): bump patch versions across workspace`

## Summary

- 6 direct deps bumped within existing semver ranges (all compatible patch releases).
- 1 transitive bump (`rand 0.9.2 -> 0.9.4`) cleared a security advisory.
- 0 breaking changes. No source code changes required.
- Bevy held at `0.18.1`; Bevy 0.19 is pre-release (`0.19.0-rc.1`) and `bevy_ecs_tilemap` / `bevy_light_2d` have no 0.19 release yet.

## Per-crate

### `ron` 0.12 -> 0.12.1

- **Type:** compatible (used in `dialog`, `quest`)
- **Release notes:** https://github.com/ron-rs/ron/releases/tag/v0.12.1
- **Migration:** none
- **Notes:** new `type` and `schema` attributes; integer suffix parsing fix for non-decimal bases; typetag version bound fix.

### `rand` 0.10 -> 0.10.1

- **Type:** compatible (used in `dialog`, `level`)
- **Release notes:** https://github.com/rust-random/rand/releases/tag/0.10.1
- **Migration:** none. Deprecates the `log` feature (not used by this project).
- **Notes:** soundness fix for custom logger interaction (PR #1763); `make_rng` now `#[track_caller]`.

### `getrandom` 0.4 -> 0.4.2

- **Type:** compatible (used in `dialog`, `level`, both with `wasm_js` feature)
- **Release notes:** https://github.com/rust-random/getrandom/blob/master/CHANGELOG.md
- **Migration:** none
- **Notes:** patch chain.

### `web-sys` 0.3.95 -> 0.3.98

- **Type:** compatible (used in `save`, wasm-only)
- **Release notes:** ships with `wasm-bindgen` releases, see below
- **Migration:** none
- **Notes:** added `ViewTransition` level-2 bindings (not used by this project).

### `wasm-bindgen` 0.2.118 -> 0.2.121

- **Type:** compatible (used in `save`, wasm-only)
- **Release notes:** https://github.com/wasm-bindgen/wasm-bindgen/releases
  - 0.2.119: wasm64 target support, Promise tuple ergonomics
  - 0.2.120: `slice_to_array` attribute, `AggregateError` bindings, struct inheritance via `extends`, `FinalizationRegistry` bindings
  - 0.2.121: (latest at upgrade time)
- **Migration:** none. All new APIs are opt-in.

### `husky-rs` 0.3.2 -> 0.3.3 (dev-dep)

- **Type:** compatible patch (used by `evergreen_main` for git hook install)
- **Release notes:** https://crates.io/crates/husky-rs/0.3.3
- **Migration:** none

## Transitive bump worth noting

### `rand` (transitive copy via `bevy_math`) 0.9.2 -> 0.9.4

- Auto-resolved by `cargo update`.
- Cleared **RUSTSEC-2026-0097** ("Rand is unsound with a custom logger using `rand::rng()`").
- No action needed in our code.

## Skipped

### `bevy` 0.18.1 (latest 0.19.0-rc.1)

- **Reason:** pre-release. Plugin ecosystem coupling: `bevy_ecs_tilemap 0.18.1` and `bevy_light_2d 0.9.0` have no 0.19 release published. Defer until Bevy 0.19 stable lands and plugins follow.
- **When ready:** the `0.18 -> 0.19` migration will require a dedicated session because Bevy minor bumps historically touch every system in the workspace (cf. existing memory entries for the 0.17 -> 0.18 migration).

### `paste` 1.0.15 (unmaintained)

- **Advisory:** RUSTSEC-2024-0436 -- crate no longer maintained
- **Reason:** transitive via `metal -> wgpu-hal -> wgpu -> bevy_render`. Cannot remove without `wgpu` upgrading away from `paste`, which in turn rides Bevy.
- **Action:** track upstream; will resolve when Bevy bumps `wgpu`.

## Verification

| Check | Result |
|-------|--------|
| `cargo build --target x86_64-unknown-linux-gnu` | green |
| `cargo build` (wasm32) | green |
| `cargo clippy --target x86_64-unknown-linux-gnu --workspace` (lib only, project's `just lint`) | green (2 pre-existing warnings) |
| `cargo test --target x86_64-unknown-linux-gnu --workspace` | 4 pre-existing failures in `lighting::ambient` -- **not introduced by this upgrade** (verified by re-running tests on the pre-upgrade tree; same failures) |
| `cargo audit` | 1 warning remaining (`paste` unmaintained, transitive, see above); was 2 before |

## Follow-ups

- [ ] Fix the 4 pre-existing `lighting::ambient` test failures: tests compare `Color::Srgba(...)` against `Color::LinearRgba(...)` with `assert_eq!`, but `target_for_hour` always returns a linear color via `lerp_linear_color`. Fix tests to compare in linear space, or have `target_for_hour` short-circuit at exact anchor matches.
- [ ] Once Bevy 0.19 stable + ecosystem plugins are released, run `/upgrade-deps` again with major-bump scope opted in; will clear the remaining `paste` advisory.
- [ ] Repo `target/` directory was 188 GB before this upgrade (108 GB wasm + 70 GB native); needed to clean native to fit. Consider running `cargo clean` periodically.
