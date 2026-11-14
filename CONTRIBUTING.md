# Contributing

1. Fork, branch, small commits.
2. `scripts/dev-check.sh` must pass locally (fmt, clippy -D warnings,
   tests, compat-matrix sync).
3. Commits that change behavior must update documentation/ and the
   relevant milestone notes.
4. No compatibility claims without a matrix entry and a test.
