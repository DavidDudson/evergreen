# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview


## Build & Development Commands

### Running

```bash
trunk serve
```

The project uses [Trunk](https://trunkrs.dev/) as its WASM build tool and dev server. Running `trunk serve` builds the project and starts a dev server at `http://127.0.0.1:8080` with automatic rebuilds on file changes.

### Building

```bash
cargo build                           # Default WASM build
cargo build --release                 # Optimized release build
```

### Testing

```bash
cargo test                            # Run all tests
```

### Linting & Formatting

```bash
cargo clippy            n linter
cargo fmt                             # Format code
cargo fmt -- --check                  # Check formatting without modifying
cargo fix --allow-staged --allow-dirty  # Auto-fix lint issues
```

The pre-commit hook (`.husky/hooks/pre-commit`) automatically runs `cargo fix` and `cargo fmt` on commit.

### Prerequisites
- Rust nightly toolchain (specified in `rust-toolchain.toml`)
- WASM target: `rustup target add wasm32-unknown-unknown`
- Trunk: `cargo install trunk`
- OS-specific Bevy dependencies (see [Bevy setup guide](https://bevyengine.org/learn/quick-start/getting-started/setup/))
  - Linux: `libasound2-dev`, `libudev-dev`, `pkg-config`

## Skills

This project has custom skills to accelerate common tasks. Use them proactively:

- **`/create-dialog`** — Add dialogue scripts, barks, or NPC `Talker` entities. Covers locale keys, RON asset format, flag-gated branching, and `BarkPool` setup.
- **`/bevy-18`** — Bevy 0.18 API reference. **Use this skill before writing any Bevy system, component, event, asset loader, or plugin.** Covers renamed APIs (`Message` instead of `Event`, `MessageReader`/`MessageWriter`, `ChildOf`, etc.), WASM patterns, and common pitfalls.
- **`/wiki-update`** — Push research files to the EvergreenNova fandom wiki. **Use this skill for any wiki write operation.** Enforces three rules: (1) edits to human-owned pages go under a `Bot Conjecture` section (treated as unverified); (2) new bot-created pages get a warning banner; (3) every edit requires explicit manual approval before the API is called. Also documents pywikibot/mwclient for complex bulk operations.
- **`/wiki-enrich`** — Enrich an existing wiki character page with transcript-sourced content. Fetches the existing page, analyses raw transcripts for the character, and appends new History, Quotes, Relationships, Birth/Death, and Appearances sections under `Bot Conjecture`. Never replaces existing content. Includes YouTube timestamps for key moments.
- **`/wiki-create-character`** — Create a new character page on the wiki from session transcripts and research files. For characters that don't have a wiki page yet. Uses `{{Bot-created}}` banner, `{{Evergreen_NPC}}` infobox, and full page structure (Description, History, Relationships, Quotes, Appearances).

### Keybinds

All player input should go through `Res<Keybinds>` rather than hardcoded `KeyCode` values:

```rust
fn my_system(keyboard: Res<ButtonInput<KeyCode>>, bindings: Res<Keybinds>) {
    if keyboard.just_pressed(bindings.key(Action::Interact)) { ... }
}
```

- Add new actions to `Action` in `keybinds/src/action.rs` — include a default binding in `Keybinds::default()` and a display label in `Action::label()`
- Add the new `KeyCode` variant to `keycode_name`/`keycode_from_name` in `keybinds/src/serialize.rs` and `keycode_label` in `ui/src/keybind_screen.rs`
- **Do not hardcode `KeyCode` values** in gameplay systems — always look up via `Keybinds`
- Bindings persist automatically via the `save` crate: WASM → `localStorage["evergreen.save"]`, native → `./evergreen_saves/evergreen.save.json`
- **Do not add storage logic to `keybinds`** — the `save` crate owns all I/O. To persist new data, add a field to `save/src/file.rs::SaveFile` and wire it in `save/src/plugin.rs`.
- **`GameSettings`** lives in `models/src/settings.rs` (master/bgm/sfx volumes 0–10, fullscreen bool). Settings are persisted via `save` and synced to Bevy's `Window` by `settings_screen::apply_fullscreen`. Wire audio volume sync in `settings_screen` when audio is added.

## MCP Server Usage

This project benefits from several MCP servers available in the environment. Use these tools proactively:

**Git & GitHub:**
- Use the `git` MCP server for advanced git operations (viewing history, diffs, branch management)
- Use the `github` MCP server for creating/managing issues, pull requests, and viewing repository metadata
- Prefer MCP tools over bash commands for git operations when available

**Fetch:**
- Use the `fetch` MCP server to retrieve Bevy documentation, crates.io package info, or external resources
- Helpful for checking latest Bevy/Rapier API docs or finding solutions to common patterns

**Memory:**
- Use the `memory` MCP server to persist important project decisions, architecture notes, or frequently needed context
- Store information about why certain architectural choices were made
- Remember common issues and their solutions across sessions

**Context7:**
- Use for enhanced context management when working on complex, multi-file changes
- Helpful for tracking related changes across the workspace crates

## Architecture

### Workspace Structure

The project uses a Cargo workspace with crates organized by domain responsibility:

**Core Application:**
- **evergreen_main**: Entry point that orchestrates all plugins and initializes the Bevy app

**Shared Data:**
- **models**: Core components and types used across all crates
  - `Health`, `Attack`, `Speed`, `Hardness`, `Distance`, `Name`
  - `GameState` enum - MainMenu, Playing, Paused, GameOver, Dialogue, LorePage, KeybindConfig
  - Palette colors (`models/src/palette.rs`) — all colors defined here

**Game Systems (Plugins):**
- **combat**: Combat mechanics with message-driven damage system
- **level**: Tilemap, scenery, terrain, area streaming
- **camera**: Camera pan/zoom
- **player**: Player sprite, 8-direction animation, movement, collision
- **dialog**: NPC dialogue system — scripted trees, bark pools, lore book, locale/i18n
  - Scripts: `assets/dialogue/scripts/*.dialog.ron`
  - Barks: `assets/dialogue/barks/*.dialog.ron`
  - Locale: `assets/locale/en-US.locale.ron`
- **keybinds**: Configurable keybinds with in-memory remap logic; persistence delegated to `save`
- **save**: Single unified save file (`evergreen.save.json` / `localStorage`). Owns all platform I/O. Loads into `Keybinds` + `LoreBook` on `PreStartup`; saves on any change in `PostUpdate`.
- **ui**: All menu/HUD systems (MainMenu, HUD, PauseMenu, GameOverMenu, DialogBox, LorePage, KeybindConfigScreen)
- **diagnostics**: Debug utilities

### Dependency Flow
```
evergreen_main (orchestrator)
├── models (shared types - no internal dependencies)
└── diagnostics (standalone)
```

### Key Architectural Patterns

**-Architre:**
Each domain exports a Bevy `Plugin` that registers its systems and components. The main app in `evergreen_main` composes these plugins.

**Event-Driven Combat:**
```
Collision/Range Detection → DamageEvent → Health Update → DeathEvent → Despawn/GameOver
```
- `combat::detect_attacks` - Checks range, emits DamageEvent
- `combat::handle_collisions` - Converts Rapier ContactForceEvent to DamageEvent based on hardness
- `combat::apply_damage` - Processes DamageEvent, updates Health, emits DeathEvent
- `combat::handle_deaths` - Despawns entities or triggers GameOver

**State Management:**
Systems run conditionally based on `GameState`:
- `Update.run_if(in_state(Playing))` - Active gameplay systems
- `OnExit(Playing).run_if(not(in_state(Paused)))` - Cleanup (conditional to preserve state when paused)

**ECS Component Composition:**
Entities are cll, focused components. Use `#[require(...)]` for component dependencies.

**Physics Integration:**
Bevy Rapier2D (2D rigid bodies, 100 pixels per meter):
- ContactForceEvent drives collision damage
- Kinematic controls for peasant dragging

### Game State Flow
```
MainMenu → Playing ↔ Paused → GameOver
MainMenu → LorePage → MainMenu
Playing  → Dialogue → Playing
Playing  → KeybindConfig → Playing  (or from PauseMenu)
```
- MainMenu: Initial state
- Playing: Active gameplay
- Paused: Auto-triggered on window focus loss
- Dialogue: NPC conversation (world frozen)
- LorePage: Lore browser (from main menu)
- KeybindConfig: Key remapping UI (accessible from pause menu)

## Code Conventions

- Strong typing with newtypes for domain clarity (Health(u16), Distance(u16), Speed(u16))
- Use `derive_more` for ergonomic arithmetic operations on newtypes
- Each crate exports one main plugin (`CastlePlugin`, `EnemyPlugin`, etc.)
- Event-driven for decoupled systems (DamageEvent, DeathEvent)
- Query-based system parameters for efficient ECS access
- All colors must be defined in `models/src/palette.rs` — inline `Color::srgb()` etc. is banned by `clippy::disallowed_methods`
- **No magic numbers** — All numeric literals must be assigned to named constants
  - Constants with a corresponding newtype must use it: `const MELEE_RANGE: Distance = Distance(200);` not `const MELEE_RANGE: u16 = 200;`
  - Bare numeric constants must include units in the name: `TILE_SIZE_PX`, `SCREEN_WIDTH_PX`, `BUTTON_PADDING_H_PX`
  - If a constant would end up as `String`, `usize`, or `isize`, consider creating a newtype with `From`/`TryFrom` impls instead

### Recommended Patterns

- **Functional style** — Prefer iterators, `map`, `flat_map`, `filter`, `fold` over imperative loops. Use combinators (`and_then`, `map_or`, `ok_or`) over `if let`/`match` chains where it improves clarity.
- **Trait-based design** — Define behavior through traits (typeclass pattern). Implement `From`/`TryFrom`, `Display`, `Default`, and domain-specific traits to keep logic polymorphic and decoupled.
- **Small files** — No file should exceed 300 lines. If a file grows beyond this, split it into focused modules with clear responsibilities. Each module should do one thing well.
- **Clear architectural boundaries** — Keep crate and module boundaries tight. A module should have a single reason to change. Prefer many small modules over few large ones.
- **Plugin files are wiring only** — A `plugin.rs` file should contain only the `Plugin` impl and system registration. All components, systems, helpers, and constants belong in separate domain modules imported by the plugin. This keeps the plugin a clear table of contents for the crate.

### Banned Patterns (enforced by workspace clippy lints)

- **No `unwrap()`** — Use `expect()` with a descriptive message for cases that are logically infallible, or propagate errors with `?`/`map`/`and_then`. `clippy::unwrap_used` is set to `deny`.
- **No `as` casts** — Use `From`/`Into` for infallible conversions, `TryFrom`/`TryInto` for fallible ones. If `as` is truly unavoidable (e.g. const context, no `From` impl exists), add a local `#[allow(clippy::as_conversions)]` with a comment explaining why. `clippy::as_conversions` is set to `deny`.
- **No inline color constructors** — `Color::srgb()`, `Color::srgba()`, `Color::linear_rgb()`, `Color::linear_rgba()` are banned via `clippy::disallowed_methods`. Define all colors as named constants in `models/src/palette.rs` (the only file with `#[allow(clippy::disallowed_methods)]`).
- All workspace crates inherit these lints via `[lints] workspace = true` in their `Cargo.toml`.