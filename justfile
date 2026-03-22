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

# ── Wiki ─────────────────────────────────────────────────────────────────────

# List all research files mapped to wiki pages
wiki-list:
    ./scripts/wiki_push.sh list

# Push all research files to the fandom wiki (requires 1Password)
wiki-push:
    ./scripts/wiki_push.sh push-all

# Push a single file: just wiki-push-one "Page Title" research/path/to/file.md
wiki-push-one title file:
    ./scripts/wiki_push.sh push "{{title}}" "{{file}}"

# Show diff between a local file and current wiki content
wiki-diff title file:
    ./scripts/wiki_push.sh diff "{{title}}" "{{file}}"
