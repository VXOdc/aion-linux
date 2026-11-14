#!/usr/bin/env bash
# Full local gate — must pass before every commit.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT/core"
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test --locked
bash "$ROOT/scripts/check-compat-matrix.sh"
echo "aion: all checks passed"
