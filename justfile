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
