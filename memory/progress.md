# Atomic Progress Log

Your temporal anchor. Tick atomic tasks as you complete them. Never mark a
task done unless `memory/verify.md` criteria are met.

The `state-enforcement.sh` hook blocks task completion if source files
changed but this file wasn't updated.

## In Progress

- [ ] Slice 5 publish/Engine loop — local VP-Tree publish gate is green;
  remote steps still require explicit approval: push branch/PR, merge, GHCR
  `v0.5.0`, `submission` update, Rinha Engine preview.

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
- [x] Slice 4.4 — `Dockerfile` written: stages chef → planner → builder →
  preprocessor → runtime (distroless/cc-debian12:nonroot). `data/*.bin` baked
  at `/data/`, `HEALTHCHECK` invokes `/app/api --healthcheck` every 2 s with
  20 s start-period. ADR-0001/ADR-0002 referenced in header. **Live build
  validation deferred to 4.6** (Docker daemon offline at commit time).
- [x] Slice 4.5 — `.dockerignore` written. Excludes target/, data/, bench/,
  test/, agent/harness dirs, docs, *.md, .git/. Keeps Cargo.toml, Cargo.lock,
  .cargo/, src/, resources/references.json.gz (other resources/*.json files
  also excluded — they are not loaded at runtime, mcc_risk is a const table).
- [x] Slice 4.6 — Local Tier 1 DoD validated on darwin/arm64. `docker buildx
  build --platform linux/arm64 -t rinha-fraud-rust:local --load .` produced
  a 116 MB image (87 MB data, 922 KB binary, ~25 MB distroless base). Local
  compose (2× api 0.45/170MB + nginx 0.10/10MB, agreed env vars + nginx tune)
  came up `--wait` healthy in 6 s. `k6 run test/smoke.js` 5/5 pass, p95 33 ms.
  `k6 run test/test.js` ran 120 s ramp 1→900 rps, 15631 iterations completed,
  **0 FP, 0 FN**, 9884 client-side timeouts (expected: 0.45 CPU × ARM emul
  ≪ Mac Mini 2014). No OOM, no restarts. Post-load RSS api1=7 MiB,
  api2=27 MiB (cgroup v2 + overlayfs apparently dedup'd mmap pages across
  containers — bonus, not assumed by ADR-0001). Score (-6000) is meaningless
  for Slice 4 DoD: Tier 1 is "compose holds together", not performance.
- [x] Slice 4.7 — `bench/slice-3/...` already tracked (commit `994395a` in
  Slice 3 baseline session). Grilling decision (track, not ignore) formalized
  retroactively. No further action.
- [x] Slice 4.8 — PR #8 `feat/slice-4-topology` merged (squash `4266d24`).
- [x] Slice 4.9 — `v0.4.0` pushed to GHCR (amd64, QEMU). Needed hotfix
  PR #9 (`3c18c28`, `process::exit(0)` to dodge QEMU+AVX2 destructor
  segfault in preprocess) before the build succeeded.
- [x] Slice 4.10 — `submission` `docker-compose.yml` updated (`fb90b82`):
  `:v0.4.0`, split 0.10/10MB + 0.45/170MB×2, env vars, `service_healthy`.
- [x] Slice 4.11 — `submission` `nginx.conf` updated (`fb90b82`):
  worker_processes 1, epoll, access_log off, server_tokens off, keepalive 32.
- [x] Slice 4.12 — Tier 1 via GHCR image: compose `--wait` healthy 6 s,
  smoke 5/5, test.js 14515 iters, 0 FP/0 FN, no OOM. (QEMU arm64 — not the
  score, just structural.)
- [x] Slice 4.13 — DoD measurement DONE. Upstream participant PR #4583
  merged (`c89739c`); GHCR `v0.4.0` made public (was private by default;
  follow-up LABEL fix in PR #10 prevents the regression on v0.5.0+).
  Prévia issue #4586 → Engine on Mac Mini Late 2014:
  **final_score=-6000** (p99=2002.11ms cut + failure_rate=89.13% cut),
  FP=0, FN=0, http_errors=12689. Artifacts in
  `bench/slice-4/20260515-203342-engine-fb90b82/`. **Slice 4 CLOSED.**

## Completed (this session)

- [x] Slice 5 grilling + issue #11 created — locked exact equivalence first,
  same canonical top-5/`approved`/`fraud_score`, tie-break by
  `(distance, original_index)`, squared Euclidean over `i16`, build-time baked
  VP-Tree, brute force explicit validation mode, no silent fallback.
- [x] Slice 5 VP-Tree implementation in `feat/slice-5-vptree` worktree —
  `src/lib.rs` exposes shared modules; `src/index.rs` now supports
  `SearchMode::{VpTree, BruteForce}`, canonical `Neighbor` top-5, in-repo
  VP-Tree build/load/search, `u64` distances/thresholds, `vptree.nodes.bin` +
  `vptree.metadata.json` checksum validation; `src/bin/preprocess.rs` writes
  VP-Tree artifacts during build-time preprocess; `src/bin/validate_equivalence.rs`
  validates all official test-data outputs plus top-5 sample.
- [x] Slice 5 local validation — `cargo check --locked`, `cargo fmt --check`,
  `cargo clippy --locked --all-targets --all-features -- -D warnings`,
  `cargo test --locked`; `cargo run --release --bin preprocess` generated
  refs/labels/metadata/VP-Tree in 3.2s; `validate_equivalence data
  test/test-data.json 512` passed for 54,100 entries + 512 canonical top-5
  comparisons; release server `k6 run test/smoke.js` passed 5/5 and
  `k6 run test/test.js` produced p99=3.16ms, HTTP errors=0, FP=0, FN=0,
  local final_score=5500.89. Docker arm64 image `rinha-fraud-rust:slice5-local`
  rebuilt after the `u64`/20-byte-node format fix; container healthy and smoke passed.
  Release process RSS after load: ~146.8 MiB. Artifacts:
  `bench/slice-5/20260516-121352-local-vptree-u64-node20/`.

- [x] Slice 5 exactness hardening (`feat/slice-5-vptree-exact`, review
  follow-up) — VP-Tree pruning moved from `f64::sqrt` (epsilon ≪ sqrt
  rounding error → not provably exact) to integer-exact `sqrt_le_sum`
  (`i128`/`u128`, no float on search path); `validate_equivalence` gained a
  deterministic SplitMix64 differential fuzz over reference-derived
  (exact/near/far-perturbed) + uniform-random queries; ADR-0003 documents
  integer pruning, the `count < K` loud-fail behavior change, and the
  validation layers. Verified on full 3,000,000-ref dataset: 51 unit tests
  pass, clippy clean (`-D warnings`), `validate_equivalence data
  test/test-data.json 512 5000` → 0 mismatches (54,100 official outputs +
  512 strided top-5 + 5000 random differential).

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
