# Atomic Progress Log

Your temporal anchor. Tick atomic tasks as you complete them. Never mark a
task done unless `memory/verify.md` criteria are met.

The `state-enforcement.sh` hook blocks task completion if source files
changed but this file wasn't updated.

## In Progress

- [ ] Open PR for Slice 1 + setup (this branch -> main).

## Completed (this session)

- [x] Configure `agent-md.toml`.
- [x] Enable Codex hooks in `~/.codex/config.toml`.
- [x] Enable git hook fallback (`core.hooksPath = .githooks`).
- [x] Seed agent-md memory files from local official docs.
- [x] Trade-off analysis closed (axum, i16, build-time preprocess, brute SIMD,
  nginx, GHCR per-slice tags, MIT license,
  `obrunogonzaga/rinha-backend-2026-rust`). Documented in vault
  `07-trade-offs-iniciais.md` and `08-decisao-preprocess.md`.
- [x] Public GitHub repo created and `main` pushed.
- [x] Orphan `submission` branch created and pushed with skeleton
  (`docker-compose.yml`, `nginx.conf`, `info.json`, `README.md`).
- [x] Slice 1 — `axum` + `tokio` + `serde` added to `Cargo.toml`.
- [x] Slice 1 — `GET /ready` returning `{"ready": true}` implemented.
- [x] Slice 1 — `POST /fraud-score` placeholder returning
  `{"approved": true, "fraud_score": 0.0}` implemented.
- [x] Slice 1 — service binds to `0.0.0.0:9999`.
- [x] Slice 1 — unit tests cover `/ready` and `/fraud-score` shapes (2 passing).
- [x] Slice 1 — service ran locally and `k6 run test/smoke.js` passed
  (5/5 iterations, max 247us, p95 218us, zero HTTP failures).
- [x] Slice 1 baseline recorded in vault `09-baseline-medicoes.md`.
- [x] `LICENSE` (MIT) added at repo root.
- [x] `cargo check --locked`, `cargo fmt --check`,
  `cargo clippy --locked --all-targets --all-features -- -D warnings`,
  `cargo test --locked` all pass.

## Backlog (next up)

- [ ] Slice 2a — vectorization with official fixtures.
- [ ] Slice 2b — preprocessor binary streaming `references.json.gz` to `i16`
  binary.
- [ ] Slice 3 — brute-force SIMD search.
- [ ] Slice 4 — multi-stage Dockerfile, GHCR push, submission compose.

## Blocked

<!--
- [ ] <task> — waiting on: <reason or person>
-->
