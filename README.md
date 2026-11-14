# AION Linux

An AI-native Linux distribution built on the Arch Linux ecosystem, for
AArch64 and x86-64. One operating system for running, developing, and
serving AI — from a Raspberry Pi 5 to a multi-GPU workstation.

```
      _    _ _
  ___| |__(_) |___
 / __| '_ \ | / __|
| (_| | | | | \__ \
 \___|_| |_|_|___/
```

**Status: Milestone 3 in progress — hardware detection and AION CLI.**
See the [milestones](documentation/milestones.md) and
[architecture overview](ARCHITECTURE.md).

## What works today

This repository currently contains working code for:

- **`aion` CLI** — `system info`, `hardware detect`, `hardware benchmark`,
  `model list`, `runtime list`, `service list`, `doctor`. Unimplemented
  commands say so honestly instead of faking success.
- **`aiond` daemon** — gathers the hardware report into
  `/run/aion/hardware.json`, independently of any graphical session.
- **GGUF parser** — reads model headers (name, architecture, context
  length) with no external dependencies.
- **Runtime compatibility matrix** — data-driven; a runtime is only
  advertised for hardware combinations the matrix has been tested on.
- **13 unit/integration tests**, zero clippy warnings, reproducible
  `Cargo.lock`.

## Try the CLI on any Linux

```
cd core
cargo run -- system info
cargo run -- hardware detect
cargo run -- runtime list
cargo run -- doctor
```

## Repository layout

```
core/            Rust workspace: aion-core, aion-cli, aion-hardware,
                 aion-models, aion-runtime, aion-types
architectures/   Architecture-specific build configs (x86_64, aarch64)
applications/    Desktop apps (Model Center, AI Studio, Agent Hub, ...)
services/        systemd units
packages/        PKGBUILDs for the aion-core and edition metapackages
repositories/    Package repository tooling and signing policy
installer/       ISO installer profiles (archiso-based)
build/           Image build pipeline
tests/           System-level test plans and scripts
documentation/   Architecture, decisions (ADRs), milestones
branding/        Logo, palette
infrastructure/ CI pipelines (separate x86_64 and aarch64)
scripts/        Developer scripts
data/           Compatibility matrices (YAML, source of truth)
```

## Development rules (for humans and AI agents)

Read [ARCHITECTURE.md](ARCHITECTURE.md) before writing code. Inspect
existing code before modifying it. Implement in small, tested increments.
Never claim a build succeeded without running it. Never fabricate test
results. Report blockers honestly. Full rules:
[documentation/development.md](documentation/development.md).

## License

MIT for AION-authored code. Third-party components keep their own
licenses; the distribution as a whole respects each component's terms.
See [LICENSES.md](LICENSES.md).
