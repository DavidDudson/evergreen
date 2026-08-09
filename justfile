# Evergreen task runner
# Usage: just <recipe>

# List available recipes
default:
    @just --list

# ── Dev ──────────────────────────────────────────────────────────────────────

# Run dev server
serve:
    trunk serve

# Build (debug)
build:
    cargo build

# Build (release)
build-release:
    cargo build --release

# Run all tests
test:
    cargo test

# Lint
lint:
    cargo clippy

# Format
fmt:
    cargo fmt

# Auto-fix lint issues
fix:
    cargo fix --allow-staged --allow-dirty && cargo fmt

# ── Art ──────────────────────────────────────────────────────────────────────

# Assemble frames into a sprite sheet (see .claude/skills/art-animation)
sheet *ARGS:
    uv run scripts/build_sheet.py {{ARGS}}

# Check committed sheets still match the grids the engine declares
sheet-check:
    uv run scripts/build_sheet.py --verify --layout npc assets/sprites/npc/*_sheet.webp
    uv run scripts/build_sheet.py --verify --layout enemy assets/sprites/enemies/*_sheet.webp
    uv run scripts/build_sheet.py --verify --layout player assets/sprites/player/*_sheet.webp
