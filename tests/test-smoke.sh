#!/usr/bin/env bash
# System smoke test — runs on a live AION system (or any Linux with
# the aion binary). Milestone 3 subset.
set -euo pipefail
BIN="${AION_BIN:-aion}"
$BIN system info
$BIN hardware detect | python3 -c 'import json,sys; r=json.load(sys.stdin); assert r["cpu"]["threads"]>0 and r["memory"]["total_bytes"]>0'
$BIN runtime list
$BIN doctor --json | python3 -c 'import json,sys; d=json.load(sys.stdin); assert "cpu" in d and "memory" in d'
$BIN model list
echo "SMOKE OK"
