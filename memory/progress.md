# Atomic Progress Log

Your temporal anchor. Tick atomic tasks as you complete them. Never mark a
task done unless `memory/verify.md` criteria are met.

The `state-enforcement.sh` hook blocks task completion if source files
changed but this file wasn't updated.

## In Progress

_(empty — Slice 3 closed in PR #4. Next vertical slice is Slice 4.)_

## Backlog (next up)

- [ ] Slice 4 — multi-stage Dockerfile (`preprocess` runs in builder, runtime
  ships binary + `data/*.bin`).
- [ ] Slice 4 — `docker-compose.yml` with nginx + api1 + api2, limits ≤ 1.5
  CPU and 350 MB total, bridge network, non-privileged.
- [ ] Slice 4 — `k6 run test/test.js` against compose; record p99, FP/FN,
  HTTP errors, RAM per container in `vault 09-baseline-medicoes.md`.
- [ ] Slice 4 — push image to GHCR as `:v0.4.0`.
- [ ] Slice 4 — submission branch updated to the GHCR tag.

## Completed (this session)

- [x] PR #4 (Slice 3) merged into `main` at `63b9f34` — brute-force scan
  over `i16` references via `memmap2::Mmap`, top-K on a stack array,
  `arg_max` instead of a heap, i64 accumulator (i32 overflow risk
  calculated). Doc examples (`tx-1329056812` legit, `tx-3330991687` fraud)
  pass byte-for-byte. `k6 smoke` p95 = 18.14 ms on darwin/arm64; NOT the
  official Linux 1-CPU number, that comes from Slice 4. 26/26 tests.
  Details in vault `13-slice-3-busca.md`.
- [x] PR #3 (Slice 2b) merged into `main` at `3724c46` — `preprocess` bin
  reads `resources/references.json.gz`, quantizes to `i16` (scale 10000),
  writes `data/refs.i16.bin` (84 MB) + `data/labels.bin` (3 MB) +
  `data/metadata.json`. SHA-256 deterministic between runs. Wall clock
  1.83 s, single thread. Labels are 66.69% legit / 33.31% fraud.
  Details in vault `12-slice-2b-preprocess.md`.
- [x] PR #2 (Slice 2a) merged into `main` at `b6e605f` —
  `vectorize(&Payload) -> [f32; 14]`, manual ISO-8601 parser, Sakamoto
  weekday, Hinnant `days_from_civil`, `mcc_risk` const table. 10 unit
  tests cover the doc fixtures byte-for-byte at 4 dp.
- [x] Vault sync after Slice 3 — new notes `12-slice-2b-preprocess.md`
  and `13-slice-3-busca.md`; baseline + bitácora + editorial + index
  updated; cover-post4 / cover-post4a Excalidraws + PNG exports in vault.
  Post 4 (`rinha-backend-2026-quantizacao-i16`) live in production at
  https://brunogonzaga.dev/artigos/rinha-backend-2026-quantizacao-i16/.

## Completed (previous sessions)

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

## Blocked

<!--
- [ ] <task> — waiting on: <reason or person>
-->
