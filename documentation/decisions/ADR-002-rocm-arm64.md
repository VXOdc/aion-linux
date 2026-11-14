# ADR-002: AMD GPU acceleration on AArch64

Date: 2026-11-13 · Status: accepted

## Context
AMD does not ship ROCm for AArch64 hosts and has stated there are no
short-term plans to support it. Building ROCm from source on ARM64 is
not a maintainable path for a distribution.

## Decision
- ROCm/HIP/rocBLAS packages are **x86_64 only**.
- On AArch64 hosts with AMD GPUs, the supported acceleration path is
  **Vulkan** (llama.cpp and runtimes with Vulkan backends).
- `aion runtime list` and the Model Center must never offer ROCm
  backends on aarch64, enforced by data/compatibility/runtimes.yaml
  and unit tests (aion-runtime::tests::rocm_never_offered_on_aarch64).

## Consequences
- AMD GPU users on ARM64 get a clearly-labeled, functional Vulkan
  tier instead of a broken ROCm install.
- The compatibility matrix carries this rule as data, so Model Center
  inherits it without code changes.
