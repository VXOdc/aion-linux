# AION Linux Architecture

## Foundation decisions (see documentation/decisions/ for rationale)

| Decision | Choice | ADR |
|---|---|---|
| Base distribution | Arch Linux ecosystem; x86_64 from Arch proper, aarch64 from the Arch AArch64 ports effort (Armv8.2 baseline) | [ADR-001](documentation/decisions/ADR-001-arch-aarch64-base.md) |
| Init / service manager | systemd | — |
| Package manager | pacman | — |
| Desktop | KDE Plasma on Wayland | — |
| Filesystem | Btrfs (snapshots + rollback via aion-update) | — |
| Sound / network | PipeWire / NetworkManager | — |
| Core language | Rust (system), Python + TypeScript (apps where appropriate) | — |
| MAC security | AppArmor (see ADR-003) | [ADR-003](documentation/decisions/ADR-003-mac.md) |
| AMD on ARM64 | ROCm is x86_64-only; Vulkan is the ARM64 fallback | [ADR-002](documentation/decisions/ADR-002-rocm-arm64.md) |

## Core platform (core/)

```
┌─────────────────────────────────────────────────────┐
│                    applications                     │
│  Model Center · AI Studio · Agent Hub · Dashboard    │
└──────────────────────┬──────────────────────────────┘
                       │ aion-api (versioned JSON)
┌──────────────────────┴──────────────────────────────┐
│ aiond (aion-core daemon)                             │
│  ┌───────────┬────────────┬─────────┬────────────┐  │
│  │ aion-     │ aion-      │ aion-  │ aion-      │  │
│  │ hardware  │ models     │ runtime│ monitor    │  │
│  └───────────┴────────────┴─────────┴────────────┘  │
│  aion-security · aion-update · aion-agent (later)   │
└─────────────────────────────────────────────────────┘
                       │
              /run/aion/hardware.json
```

- **aion-types** — shared, semver-stable types spoken over JSON.
- **aion-hardware** — procfs/sysfs detection. No root, no external
  tools, degrades gracefully.
- **aion-models** — GGUF parsing, local model store scanning, RAM/VRAM
  estimates (labeled as estimates; swap is never treated as RAM).
- **aion-runtime** — registry of capability descriptors; the source of
  truth is `data/compatibility/runtimes.yaml`, mirrored in Rust and
  checked by CI.
- **aion-core (`aiond`)** — the daemon; runs headless, before any
  graphical session.
- **aion-cli (`aion`)** — the user-facing command line.

## Current milestone scope

Milestone 3 (hardware detection + CLI) is implemented in this repo.
Milestones 4–8 add: runtime supervision, Model Center, AI Studio,
Agent Hub, server/cluster features. Planned components
(aion-monitor, aion-security, aion-update, aion-agent, aion-api)
are documented in the PRD and will land as separate crates with the
same structure and test discipline.

## Hard rules enforced by CI

1. `cargo fmt --check`, `cargo clippy -D warnings`, `cargo test` on
   both x86_64 and aarch64.
2. The Rust runtime registry must match `data/compatibility/runtimes.yaml`.
3. No package ships with a claim of support that is not in the matrix.
4. Changelog entries required for behavior changes.
