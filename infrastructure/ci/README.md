# CI

Two pipelines, always: x86_64.yaml and aarch64.yaml (separate per PRD
section 23). AArch64 runs on native runners; QEMU is allowed only for
boot smoke tests and never for performance claims. The compat-matrix
sync check runs in both pipelines, so a matrix change that forgets the
Rust registry (or vice versa) fails CI.
