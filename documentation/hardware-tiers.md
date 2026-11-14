# Hardware support tiers

| Tier | Meaning | Testing |
|---|---|---|
| 1 | Officially supported | every CI build, real hardware |
| 2 | Community-supported | best effort, community reports |
| 3 | May boot | no guarantees |

Tier 1 (target): one NVIDIA x86-64 workstation, one AMD x86-64,
one Intel Arc, Raspberry Pi 5 / Armv8.2+ SoC boards, one aarch64
server (Ampere class). QEMU provides aarch64 boot smoke tests;
GPU inference requires Tier 1 real hardware.
