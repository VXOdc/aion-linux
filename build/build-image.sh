#!/usr/bin/env bash
# Build an AION installation image for the current architecture.
# Milestone 1 target: wraps archiso with the AION profile.
set -euo pipefail
PROFILE="${1:-installer/profiles/desktop}"
ARCH="$(uname -m)"
echo "Building AION image: profile=$PROFILE arch=$ARCH"
# Milestone 1: mkarchiso -v "$PROFILE" -o out/
echo "Not implemented yet — arrives with Milestone 1 (bootable images)."
exit 2
