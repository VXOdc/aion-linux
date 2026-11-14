# x86_64 build target

- Base: Arch Linux official repositories (extra, core).
- CPU baseline: x86-64-v2 for userland; runtimes detect and use
  AVX2/AVX-512 at runtime (never assume).
- GPU: NVIDIA (CUDA), AMD (ROCm x86_64 + Vulkan), Intel (Vulkan, oneAPI).
- Boot: systemd-boot (default) or GRUB (legacy/chainload cases).
- Kernel: linux (Arch); the aion-kernel variant with version-locked
  NVIDIA driver pairing arrives with Milestone 2 packaging.
