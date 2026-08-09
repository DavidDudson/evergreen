---
name: upgrade-deps
description: Upgrade all Rust dependencies across the workspace. Audits outdated crates with cargo tooling, plans for breaking changes via changelogs, performs the upgrade, then writes a summary with release notes and migration links. Use when user says "upgrade deps", "bump crates", "update libraries", "/upgrade-deps".
tools: Bash, Read, Edit, Write, Grep, Glob, WebFetch
---

# Upgrade Dependencies

Four phases: **brainstorm → plan → upgrade → summarize**. Do not skip ahead. Each phase has a gate the user must pass before next phase starts.

## Prereqs

Check tools exist. Install missing ones in a transient nix shell (do NOT modify shell.nix unless asked).

```bash
cargo upgrade --version   # cargo-edit (primary tool)
cargo audit --version     # cargo-audit
cargo info bevy           # builtin since cargo 1.79
cargo tree --version      # builtin
```

If missing, run via:
```bash
nix-shell -p cargo-edit cargo-audit --run "cargo upgrade --dry-run --incompatible"
```

**Note on `cargo-outdated`:** historically the go-to tool, but it has a
known unbounded-memory issue on Bevy-sized workspaces (kbknapp/cargo-outdated#383,
open since 2024). On this repo a single invocation peaks at 30–80 GB.
**Prefer `cargo upgrade --dry-run`** below; only fall back to cargo-outdated
if you specifically need its `Compat` vs `Latest` split (rare).

Workspace uses **per-crate dep versions** for everything except `bevy`, which
lives in the root `[workspace.dependencies]` table (member crates use
`bevy = { workspace = true }`). All `Cargo.toml` files under each member crate
must be inspected, plus the root. Workspace lints in root `Cargo.toml`.

**Bevy itself is out of scope for this skill.** A Bevy minor bump is a
migration, not a dependency bump, and it is gated on every `bevy_*` plugin
publishing a compatible release. Use `/upgrade-bevy` for that; it owns the
readiness check, the migration-guide scoping, and the per-crate gates.

Tests need `--target x86_64-unknown-linux-gnu` (default target is wasm). See
[[cargo_test_target]]. The native build also links wayland/alsa/udev, which
`shell.nix` does not provide, so wrap it:

```bash
nix-shell -p pkg-config wayland libxkbcommon alsa-lib udev libGL vulkan-loader \
  --run "cargo test --target x86_64-unknown-linux-gnu"
```

## Phase 1 — Brainstorm (read-only)

Goal: enumerate every dep, current vs latest, and classify upgrade risk. **No writes.**

1. List members:
   ```bash
   rg '^members' Cargo.toml -A 20
   ```
2. Show outdated table for whole workspace using `cargo upgrade --dry-run`
   (reads each member's `Cargo.toml`, queries crates.io for latest, does
   NOT re-resolve the dep graph — cheap and fast):
   ```bash
   # Compatible (patch/minor) upgrades
   cargo upgrade --dry-run > /tmp/upgrade-compat.txt

   # Plus major-version upgrades
   cargo upgrade --dry-run --incompatible > /tmp/upgrade-major.txt
   ```
   Output lists each crate with current → new version. Compatible run
   gives the low-risk set, `--incompatible` reveals the breaking set.

   ⚠️ **Do NOT use `cargo outdated --workspace`** on this repo as the
   default. It runs cargo's resolver twice (current + latest) and on
   Bevy workspaces peaks at 30–80 GB of RAM — kbknapp/cargo-outdated#383.
   Past incident: two parallel runs used 120 GB RAM + 77 GB swap and
   froze the system. If you absolutely need cargo-outdated's `Compat`
   column, run it **once**, save to a file, slice the file with
   head/tail — never run two invocations in parallel.

   For per-crate detail (very cheap, no resolver), use built-in
   `cargo info`:
   ```bash
   cargo info bevy        # shows current pinned + latest published
   cargo info serde
   ```

   ⚠️ `cargo info` reads a **cached** registry index and goes stale for months
   at a time (it reported bevy's latest as `0.19.0-rc.1` three months after
   `0.19.0` shipped). When the answer matters, query the sparse index directly:
   ```bash
   idx() { c=$1
     case ${#c} in
       1) p="1/$c";; 2) p="2/$c";;
       3) p="3/$(printf %s "$c"|cut -c1)/$c";;
       *) p="$(printf %s "$c"|cut -c1-2)/$(printf %s "$c"|cut -c3-4)/$c";;
     esac
     curl -sA "evergreen-dep-audit (davidjohndudson@gmail.com)" "https://index.crates.io/$p"; }

   idx serde | jq -r 'select(.yanked==false)|.vers' | sort -V | tail -3
   ```
   Sort with `sort -V`, not "last line" -- index order is publish order, so a
   backported `0.9.5` can appear after `0.10.1`. Versions containing `-` are
   pre-releases; skip them.

   The crates.io **JSON API** (`https://crates.io/api/v1/...`) rejects requests
   from this environment under its data-access policy. Use the sparse index.
   - Flag any row where `Latest != Project` — split into:
     - **Compatible** (`Compat != Project`): patch/minor bumps, low risk.
     - **Breaking** (`Latest != Compat`): major bumps, needs migration plan.
3. Direct vs transitive: only direct deps (declared in member `Cargo.toml`s) are upgrade targets. Transitives ride along.
4. Security audit:
   ```bash
   cargo audit
   ```
   Flag any advisory as **must-fix**, even if semver-breaking.
5. Bevy ecosystem coupling: anything `bevy_*` or `bevy-*` (e.g. `bevy_rapier2d`, `bevy_light_2d`, `bevy_ecs_tilemap`) is pinned to Bevy's minor version. If Bevy bumps, all bevy plugins must bump together to a compatible release. Check each plugin's README for the Bevy version matrix before planning.
6. Toolchain check: read `rust-toolchain.toml`. If any new dep's MSRV exceeds it, note it.

Output to user — table grouped by risk:
```
| Crate | Current | Latest | Risk | Notes |
|-------|---------|--------|------|-------|
| bevy  | 0.18.1  | 0.19.0 | BREAKING | minor bump, full migration guide |
| serde | 1.0.210 | 1.0.219 | compat | patch chain |
```

**Gate:** ask user to confirm scope. Default = "compatible + must-fix security". Major bumps only if user opts in by name.

## Phase 2 — Plan breaking changes (read-only)

For every crate in the **breaking** set the user opted into:

1. Fetch changelog / release notes. Order of preference:
   - `https://github.com/<owner>/<repo>/releases` (use WebFetch)
   - `CHANGELOG.md` in the repo
   - Migration guide if linked from release (Bevy, axum, hyper, etc. publish dedicated guides)
2. Grep the workspace for current usage of the crate's public API to scope the blast radius:
   ```bash
   rg -l 'use bevy_rapier2d::' --type rust
   rg '\b<symbol>\b' --type rust
   ```
3. Cross-reference with project memory. Existing migration notes live in:
   - [[bevy_18_hdr_marker]], [[bevy_18_sprite_render]], [[bevy_light_2d_api]], [[bevy_system_tuple_limit]]
   - `.claude/skills/bevy-18/SKILL.md` — update this if Bevy bumps.
4. Write a migration checklist per breaking crate:
   ```
   ### bevy 0.18 → 0.19
   - [ ] Renamed: `FooBar` → `BazQux` (5 call sites in player/, ui/)
   - [ ] Removed: `Camera::hdr` field (already migrated, see memory)
   - [ ] New required component on `Sprite`
   - Refs: https://bevyengine.org/learn/migration-guides/0-18-to-0-19/
   ```

**Gate:** show the per-crate checklists to user. They approve / amend / drop crates.

## Phase 3 — Perform upgrade

Order matters. Do upgrades **one logical group at a time**, compiling after each.

1. **Branch off**:
   ```bash
   git switch -c chore/upgrade-deps-$(date +%Y%m%d)
   ```
2. **Group order** (smallest blast radius first):
   1. Patch/minor bumps for non-Bevy deps (`cargo upgrade --compatible` per crate, or `--incompatible` for opted-in majors)
   2. Security must-fixes
   3. Bevy plugins (bumped together with Bevy itself if Bevy is being upgraded)
   4. Bevy core last (touches every crate)
3. Per group:
   ```bash
   cargo upgrade -p <crate>@<version> --workspace
   cargo update -p <crate>
   cargo build --target x86_64-unknown-linux-gnu
   ```
   Then, once the direct bumps are green, run a **bare `cargo update`** to move
   transitives within their existing semver ranges, and re-run `cargo audit`.
   This is usually where the advisories actually clear: most RUSTSEC hits on
   this repo are transitive (through `wgpu`, `winit`, `bevy_asset`), and no
   `Cargo.toml` edit reaches them. On 2026-08-09 this single step cleared two
   7.5-high `quick-xml` advisories plus three unsound/yanked warnings.
   Rebuild and re-test afterwards -- it moves ~90 crates.
   - Fix compile errors using the checklist from Phase 2.
   - Re-run `cargo build` until green.
   - Then: `cargo clippy --target x86_64-unknown-linux-gnu -- -D warnings`
   - Then: `cargo test --target x86_64-unknown-linux-gnu`
   - Then: `cargo build` (wasm default target) to catch wasm-only breakage.
4. Commit per group with conventional message:
   ```
   chore(deps): bump <crate> <old> -> <new>
   ```
5. If a group blocks (API removed, no replacement): stop, ask user. Do not invent workarounds.

**Gate:** all groups green on `build + clippy + test + wasm build`. If `trunk serve` is feasible, smoke-test in browser per project rule for UI changes.

## Phase 4 — Summary

Write `UPGRADE_NOTES.md` at repo root (overwrite if exists). Format:

```markdown
# Dependency Upgrade — <YYYY-MM-DD>

## Summary
- N crates bumped (X compatible, Y breaking, Z security)
- Bevy: <old> → <new> (if applicable)
- Net lockfile delta: cargo update touched M transitive crates

## Per-crate

### <crate> <old> → <new>
- **Type:** compatible | breaking | security
- **Release notes:** <url>
- **Migration guide:** <url or "none">
- **Code changes:** <files touched, brief>
- **Notes:** <gotchas, follow-ups>

## Skipped
- <crate> <current> (latest <X>) — reason: <e.g. plugin not yet compatible with Bevy 0.19>

## Follow-ups
- [ ] <e.g. update .claude/skills/bevy-18 for renamed APIs>
- [ ] <e.g. revisit `foo` once upstream tags v2.0>
```

Also:
- Update memory files for any API renames discovered (use `revise-claude-md` skill or write directly to `~/.claude/projects/-home-ddudson-repos-evergreen/memory/`).
- If Bevy bumped: update `.claude/skills/bevy-18/SKILL.md` name + content, and the memory entries that reference the old version.

Print summary table to user with PR-ready commit list.

## Anti-patterns

- ❌ Using `cargo outdated --workspace` as the default scan. It re-runs
  the cargo resolver twice (current + latest) and peaks at 30–80 GB on
  this repo (kbknapp/cargo-outdated#383). Use `cargo upgrade --dry-run`
  instead — same information, no resolver, seconds not minutes.
- ❌ Running `cargo outdated` twice in parallel (e.g. once piped to
  `head` and once to `tail`). Two in parallel will OOM the host. Past
  incident: 120 GB RAM + 77 GB swap, system frozen.
- ❌ `cargo update` without `cargo upgrade` first — only bumps lockfile within existing semver ranges, misses real upgrades.
- ❌ Bumping Bevy without bumping every `bevy_*` plugin in the same group.
- ❌ Suppressing clippy warnings to get a green build — fix the code.
- ❌ Editing `Cargo.lock` by hand.
- ❌ Touching `rust-toolchain.toml` unless an MSRV blocker forced it (and only with user approval).
- ❌ Skipping the wasm build — project ships to wasm via Trunk; native-only green is not enough.
