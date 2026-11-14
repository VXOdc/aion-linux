#!/usr/bin/env bash
# Verify the Rust runtime registry matches the YAML source of truth.
# Milestone-3 heuristic check: the set of runtime ids must be identical.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
YAML_IDS=$(grep -E '^  - id:' "$ROOT/data/compatibility/runtimes.yaml" | sed 's/.*id: *//')
RUST_IDS=$(grep -oE 'id: "[a-z.]+"' "$ROOT/core/crates/aion-runtime/src/lib.rs" | sed 's/id: *//;s/"//g')
if [ "$YAML_IDS" != "$RUST_IDS" ]; then
  echo "compatibility matrix mismatch:"; diff <(echo "$YAML_IDS") <(echo "$RUST_IDS") || true
  exit 1
fi
# ROCm must never list aarch64 (ADR-002)
if grep -A2 'backend: rocm' "$ROOT/data/compatibility/runtimes.yaml" | grep -q aarch64; then
  echo "ADR-002 violation: rocm lists aarch64"; exit 1
fi
echo "compatibility matrix OK"
