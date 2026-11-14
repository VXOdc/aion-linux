# AArch64 build target

- Base: Arch AArch64 ports tree, Armv8.2 baseline (ADR-001).
- CPU: NEON required; BF16/i8mm/SVE detected at runtime and surfaced
  in `aion hardware detect` features.
- GPU: NVIDIA SBSA (CUDA, e.g. Grace class), Vulkan for AMD/Intel.
- ROCm is NOT available on this architecture (ADR-002) — never package
  or advertise it.
- Boot: systemd-boot with UEFI; GRUB for boards that require it.
- QEMU smoke tests run in CI; real-hardware tests run on Tier 1 boards.
