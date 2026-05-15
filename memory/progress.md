# Atomic Progress Log

Your temporal anchor. Tick atomic tasks as you complete them. Never mark a
task done unless `memory/verify.md` criteria are met.

The `state-enforcement.sh` hook blocks task completion if source files
changed but this file wasn't updated.

## In Progress

_(empty — Slice 3 closed in PR #4. Next vertical slice is Slice 4.)_

## Backlog (next up)

Slice 4 task order (locked after grilling session 2026-05-15). Each item is
sized to one bounded execution pass; verify before ticking.

Main branch (PR `feat/slice-4-topology`):
- [x] Slice 4.1 — `src/healthcheck.rs` with `probe()` + `is_status_200()`;
  `fn main() -> ExitCode` branches on `--healthcheck`. 9 unit/integration tests
  (closed port → fail; 200/404/500 → expected exit code). Verified live:
  exit=0 against running server, exit=1 after kill.
- [x] Slice 4.2 — `.cargo/config.toml` created with `target-cpu=x86-64-v3`
  scoped to `x86_64-unknown-linux-gnu`. Native arm64 `cargo check`/`test`
  unaffected (35/35 still pass).
- [x] Slice 4.3 — `[profile.release]` added (`lto=fat`, `codegen-units=1`,
  `panic=abort`, `strip=true`). Release build 8 s → 17 s on M3 (LTO cost,
  accepted). Binary: rinha_backend_2026 = 816 KB; preprocess = 409 KB.
  Doc fixture `tx-1329056812` still returns `approved=true, fraud_score=0.0`.
- [ ] Slice 4.4 — Multi-stage `Dockerfile` (cargo-chef → cargo build release →
  `cargo run --bin preprocess` → runtime distroless/cc-debian12:nonroot).
  Bakes `data/*.bin` at `/data/`. Sets `HEALTHCHECK` invoking `/app/api
  --healthcheck`. ADR-0001 cross-reference in comment.
- [ ] Slice 4.5 — `.dockerignore` (excludes `target/`, `data/`, `bench/`,
  `test/`, `.claude/`, `memory/`, `official/`, `docs/`, `*.md`).
- [ ] Slice 4.6 — Local validation: `docker buildx build --platform
  linux/arm64 -t rinha-fraud-rust:local .`; `docker compose up --wait` with
  override pointing to `rinha-fraud-rust:local`; `k6 run test/smoke.js`
  passes; `k6 run test/test.js` completes without compose crash.
- [ ] Slice 4.7 — Commit `bench/slice-3/...` to repo (decision: track, not
  ignore).
- [ ] Slice 4.8 — Open PR `feat/slice-4-topology`; merge after green CI.

Post-merge, manual on dev box:
- [ ] Slice 4.9 — `docker buildx build --platform linux/amd64 -t
  ghcr.io/obrunogonzaga/rinha-fraud-rust:v0.4.0 . --push` against GHCR.
- [ ] Slice 4.10 — `submission` branch: update `docker-compose.yml` (tag
  `:v0.4.0`; budget split `nginx 0.10/10MB`, `api1/api2 0.45/170MB`; env vars
  `TOKIO_WORKER_THREADS=1`, `MALLOC_ARENA_MAX=2`, `REFS_DATA_DIR=/data`;
  `nginx.depends_on` with `condition: service_healthy` for api1/api2).
- [ ] Slice 4.11 — `submission` branch: update `nginx.conf` per Slice 4
  decision (worker_processes 1, use epoll, access_log off, server_tokens off,
  keepalive 32 in upstream).
- [ ] Slice 4.12 — DoD functional: run `docker compose up --wait` against the
  GHCR image on darwin/arm64 emulation; smoke + test.js completion.

External / measurement (closes the slice):
- [ ] Slice 4.13 — DoD measurement: open `rinha/test` issue against the
  official Rinha repo; record Engine result (`final_score`, p99, FP, FN, Err)
  in vault `09-baseline-medicoes.md`. Fallback: VPS linux/amd64 (Hetzner/DO),
  flagged as proxy if Engine unavailable.

## Completed (this session)

- [x] Slice 3 dev-box baseline captured for future comparison —
  `bench/slice-3/20260515-115444-darwin-arm64-m3-55c4889/` holds
  `metadata.json`, `results.json`, `k6-summary.json`, `k6-stdout.log`.
  Run: native release binary, single instance, no LB, no cgroup, k6
  `test/test.js` ramp 1→900 rps over 120s. Headline: p99=1257.34 ms,
  FP=0, FN=0, HTTP errors=0, TP=19170, TN=23928, dropped iterations
  =10961, final_score=2900.55, process RSS=85 MB. NOT the official
  Linux 1-CPU number (Slice 4) — it is a darwin/arm64 baseline so we
  can quantify the algorithmic vs topology delta later. `bench/`
  currently untracked; commit/ignore choice deferred to user.

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
- [x] Repo housekeeping — `.gitignore` covers `.claude/napkin.md`,
  `.claude/worktrees/`, and `memory/diagrams/`; `git rm --cached` on
  napkin so future edits stay local. `git status` is back to clean
  after this commit.

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
