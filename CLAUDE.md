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

The project uses a Cargo workspace with 9 crates organized by domain responsibility:

**Core Application:**
- **evergreen_main**: Entry point that orchestrates all plugins and initializes the Bevy app

**Shared Data:**
- **models**: Core components and types used across all crates
  - `Health`, `Attack`, `Speed`, `Hardness`, `Distance`, `Name`
  - `Draggable`, `Dragged` - UI interaction markers
  - `GameState` enum - MainMenu, Playing, Paused, GameOver

**Game Systems (Plugins):**
- **enemy**: Enemy types and movement (Peasant: 64x64, Health: 5, Speed: 100, draggable)
- **combat**: Combat mechanics with event-driven damage system
- **level**: Game setup, ground spawning, and enemy wave management (spawns every 5s)
- **camera**: Camera controls (foll--pan, scroll-to-zoom, peasant dragging)
- **ui**: Menu systems (MainMenu, HUD, PauseMenu, GameOverMenu)
- **diagnostics**: Debug utilities (Bevy diagnostics, Rapier debug rendering)

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

### Game State Flo→ Playing ↔ Paused → GameOver
```
- MainMenu: Initial state, "Start Game" button transitions to Playing
- Playing: Active gameplay
- Paused: Auto-triggered on window focus loss, freezes gameplay

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