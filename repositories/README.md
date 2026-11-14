# Package repositories

Four repos: aion-core, aion-ai, aion-hardware, aion-community
(+ aion-experimental for bleeding-edge runtimes).

- All packages signed; repo metadata verified by pacman keyring.
- Built in clean chroots (devtools), reproducibly.
- aion-hardware carries vendor driver stacks; CUDA/ROCm versions are
  never co-bundled into one installation (enforced by conflicts/provides).
