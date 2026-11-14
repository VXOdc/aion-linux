# Development guide (binding for humans and AI coding agents)

1. Read ARCHITECTURE.md before writing code.
2. Inspect existing code before modifying it.
3. Small, testable increments. Unit tests for every module.
4. `cargo fmt && cargo clippy --all-targets -- -D warnings && cargo test`
   must pass before any commit; CI enforces it on both architectures.
5. Never claim a build succeeded without executing it. Never fabricate
   test results.
6. Unimplemented commands must say so (see aion-cli `not_implemented_yet`).
7. Compatibility claims must come from data/compatibility/*.yaml —
   never from code, never from memory.
8. Estimates (RAM/VRAM) must be labeled as estimates.
9. Update documentation when behavior changes.
10. Identify unsupported hardware honestly; report blockers honestly.
