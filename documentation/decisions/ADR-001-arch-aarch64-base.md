# ADR-001: Package base for AArch64

Date: 2026-11-13 · Status: accepted

## Context
Arch Linux proper ships x86_64 only. Two candidate bases exist for
AArch64: the Arch Linux ARM community project (archlinuxarm.org) and
the official Arch ports effort (ports.archlinux.page/aarch64).

## Decision
Build AION AArch64 packages against the **official Arch AArch64 ports
tree**, which targets an Armv8.2 baseline (Raspberry Pi 5 class and
newer server platforms). Use Arch Linux ARM only as a fallback for
older pre-v8.2 hardware, treated as Tier 3 (community-supported).

## Consequences
- x86_64 and aarch64 share PKGBUILDs with architecture conditionals,
  matching the direction of the upstream ports effort.
- Devices below Armv8.2 are out of scope for official images.
- Trademark: AION is not an Arch Linux product and must not present
  itself as one; branding reviewed before any public release.
