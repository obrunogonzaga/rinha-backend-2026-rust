# Atomic Progress Log

Your temporal anchor. Tick atomic tasks as you complete them. Never mark a
task done unless `memory/verify.md` criteria are met.

The `state-enforcement.sh` hook blocks task completion if source files
changed but this file wasn't updated.

## In Progress

- [x] Slice 3 — `src/index.rs`: mmap loader for `data/refs.i16.bin` +
  `data/labels.bin`, quantize_query, brute-force top-K scan with stack
  array (no per-request alloc), Decision { approved, fraud_score }.
- [x] Slice 3 — wired `Index` into axum `AppState` (Arc<Index>) and
  updated `/fraud-score` handler to return real decision.
- [x] Slice 3 — 12 unit tests in `index::tests` (quantize match,
  sq_dist, arg_max, score thresholds 0.0/0.4/0.6/1.0, top-5 selection
  with extra refs, from_parts validation). 4 handler tests against
  synthetic `Index`. Existing vector tests intact. 26/26 green.
- [x] Slice 3 — live boot against full `data/` (3M refs):
  - `GET /ready` → 200 `{"ready":true}`.
  - Doc legit example → `{"approved":true,"fraud_score":0.0}` ✓.
  - Doc fraud example → `{"approved":false,"fraud_score":1.0}` ✓.
- [x] Slice 3 — `k6 run test/smoke.js` against live server:
  - 5/5 iterations OK, 0% HTTP failures.
  - `http_req_duration`: min=10.02 ms, p50=10.38 ms, p95=18.14 ms,
    max=20.07 ms.
  - Single VU, scalar Rust hot loop (compiler auto-vectorized i16
    distance sum), darwin/arm64.
  - Baseline before explicit SIMD / `wide` / chunking tweaks.
- [ ] Open PR for Slice 3 (this branch -> main).

## Completed (this session)

- [x] PR #3 (Slice 2b) merged into `main` at `3724c46`.
- [x] PR #2 (Slice 2a) merged into `main` at `b6e605f`.
- [x] Slice 2a — `Payload` deserialization model in `src/vector.rs`.
- [x] Slice 2a — pure `vectorize(&Payload) -> [f32; 14]` matching the 14
  dimensions in `REGRAS_DE_DETECCAO.md`.
- [x] Slice 2a — manual ISO-8601 UTC parser (`YYYY-MM-DDTHH:MM:SSZ`),
  Sakamoto weekday (seg=0…dom=6), Hinnant `days_from_civil` for minute
  diffs.
- [x] Slice 2a — `mcc_risk` table (10 entries) + 0.5 default; constants
  hardcoded, no JSON load at runtime.
- [x] Slice 2a — 10 unit tests pass: legit + fraud doc fixtures
  byte-equal at 4dp, `-1` sentinel at idx 5/6, clamp ceiling, mcc
  unknown/known, set-membership for known_merchants, weekday for
  known dates, parse_u32, with-last-tx minutes/km computation.
- [x] Slice 2a — `POST /fraud-score` now deserializes the full payload
  via serde and runs `vectorize` (response still placeholder per Slice 3
  scope). Malformed payload returns `422`.
- [x] Slice 2a — `cargo check/fmt/clippy/test` all green; `k6 run
  test/smoke.js` 5/5, p95 220µs, 0 HTTP failures.

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

## Backlog (next up)

- [ ] Slice 2b — preprocessor binary streaming `references.json.gz` to `i16`
  binary.
- [ ] Slice 3 — brute-force SIMD search.
- [ ] Slice 4 — multi-stage Dockerfile, GHCR push, submission compose.

## Blocked

<!--
- [ ] <task> — waiting on: <reason or person>
-->
