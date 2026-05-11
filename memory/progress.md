# Atomic Progress Log

Your temporal anchor. Tick atomic tasks as you complete them. Never mark a
task done unless `memory/verify.md` criteria are met.

The `state-enforcement.sh` hook blocks task completion if source files
changed but this file wasn't updated.

## In Progress

- [x] Slice 2b — `preprocess` bin: stream `resources/references.json.gz`,
  quantize 14-dim `f32` vectors to `i16` (scale 10000), emit
  `data/refs.i16.bin` + `data/labels.bin` + `data/metadata.json`.
- [x] Slice 2b — roundtrip unit test: `i16/scale → f32` error ≤ 1/scale.
- [x] Slice 2b — re-run produces byte-identical `refs.i16.bin` /
  `labels.bin` (SHA-256 stable across runs).
- [x] Slice 2b — full run on `resources/references.json.gz`:
  - `count=3_000_000`, `dims=14`, `scale=10000`.
  - Labels: 2_000_594 legit / 999_406 fraud (≈33.3% fraud).
  - `refs.i16.bin` = 84 000 000 B
    `sha256 d5beb0640d8a35657d206e2cdd372cf7b74591be23d44253b85ca7dcac337461`.
  - `labels.bin` = 3 000 000 B
    `sha256 aecc5d8a6258f66f5d55ba0973146f0cd3ef40ba09a2d06b298bab12ac9eb920`.
  - Wall clock: ~1.83 s on the dev box, single thread, release build.
  - **TODO (outside-repo)**: copy these numbers to vault
    `09-baseline-medicoes.md`.
- [ ] Open PR for Slice 2b (this branch -> main).

## Completed (this session)

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
