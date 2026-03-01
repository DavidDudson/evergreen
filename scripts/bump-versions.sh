#!/usr/bin/env bash
#
# bump-versions.sh — Bump versions for changed workspace crates using cargo-workspaces.
#
# Usage:
#   ./scripts/bump-versions.sh [major|minor|patch]
#
# Defaults to "patch" if no argument is given.
# Requires: cargo install cargo-workspaces

set -euo pipefail

# Ensure ~/.cargo/bin is on PATH (needed in git hook contexts)
export PATH="${HOME}/.cargo/bin:${PATH}"

BUMP_LEVEL="${1:-patch}"

if ! command -v cargo-workspaces &>/dev/null; then
  echo "Error: cargo-workspaces is not installed." >&2
  echo "Install it with: cargo install cargo-workspaces" >&2
  exit 1
fi

# Check if there are any changed crates to bump
if ! cargo ws changed --all 2>/dev/null | grep -q .; then
  echo "No crates changed — nothing to bump."
  exit 0
fi

echo "Bumping changed crates ($BUMP_LEVEL)..."
cargo ws version "$BUMP_LEVEL" \
  --all \
  --yes \
  --no-git-commit
