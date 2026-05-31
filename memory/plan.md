# Macro Plan

The architectural design. Updated when direction changes. Vertical slices
only — build full-stack features end-to-end, not horizontal layers.

## Current Phase

Slice 5: Exact VP-Tree search — done. First positive official score:
prévia #4880 run 1 (2026-05-17) `final_score=+3788.34` (p99=162.80ms,
failure=0%, FP/FN=0) — VP-Tree escapes the floor. But run 2 (2026-05-23) on
the SAME image returned `-6000` (p99=2002ms, failure=66.51%, FP/FN still 0).
Same image/commit → variance is pure latency under Engine-host contention,
not a code regression. Next is Slice 6: make the positive score robust to a
contended host (failure < 15% and p99 < 2000 ms even under CPU pressure).
Deadline for final submission: 2026-06-05T23:59:59-03:00.

Completion stamps:
- Slice 1 (HTTP baseline) — PR #1 merged at `f815dff`.
- Slice 2a (vectorization) — PR #2 merged at `b6e605f`.
- Slice 2b (preprocess) — PR #3 merged at `3724c46`.
- Slice 3 (brute-force search) — PR #4 merged at `63b9f34`.
- Slice 4 (submission topology) — PRs #7/#8/#9 merged; GHCR `v0.4.0`;
  submission `fb90b82`; Engine prévia issue #4586 → `final_score=-6000`.
  Artifacts: `bench/slice-4/20260515-203342-engine-fb90b82/`.
- Slice 5 (exact VP-Tree) — PR #12 merged at `20d0ffc`; GHCR `v0.5.0`;
  submission `0ecf07a`; Engine prévia issue #4880 → run 1 `+3788.34`
  (first positive), run 2 `-6000` (host-contention variance, same image).
  Artifacts: `bench/slice-5/ENGINE-ANALYSIS.md` + run1/run2 dirs.

## Closed Decisions

- HTTP stack: `axum` + `tokio`.
- Quantization: `i16` for reference vectors (`u8` reserved for Slice 5+).
- Preprocessing: build-time inside Docker multi-stage. No JSON parser at
  runtime, no startup decompression.
- Search baseline: brute-force SIMD. Upgrade path: VP-Tree → ANN (HNSW).
- Slice 5 search mode: VP-Tree is the default target; brute force remains
  available as an explicit validation mode (`SEARCH_MODE=bruteforce`). No
  silent runtime fallback if the VP-Tree index fails to load.
- Load balancer: nginx, round-robin only.
- Image registry: GHCR at `ghcr.io/obrunogonzaga/rinha-fraud-rust:v0.{slice}.{patch}`.
  Slice N publishes `v0.N.0`; in-slice patches bump the patch field. Tags before
  Slice 4 (v0.1.0..v0.3.x) intentionally do not exist — first published image
  is Slice 4 (`v0.4.0`).
- Submission strategy: live `submission` orphan branch, image-per-slice tag.
- Public repo: `obrunogonzaga/rinha-backend-2026-rust`. License MIT.

## Vertical Slices

### Slice 1: HTTP contract baseline
- API: `GET /ready`, `POST /fraud-score` on port `9999`.
- Decision: deterministic placeholder (`approved: true, fraud_score: 0.0`).
- Tests: shape unit tests + `k6 run test/smoke.js` against the running service.
- Verify: universal Cargo checks + smoke test.

### Slice 2a: Vectorization
- Pure functions producing `[f32; 14]` per `REGRAS_DE_DETECCAO.md`.
- Fixtures from official examples, including `last_transaction: null`.
- Verify: unit tests prove vector values byte-for-byte.

### Slice 2b: Preprocessor
- Separate `cargo` binary `preprocess` that streams `references.json.gz`,
  quantizes vectors to `i16`, writes `data/refs.i16.bin`,
  `data/labels.bin`, `data/metadata.json`.
- Runs inside Docker build stage. Output baked into the runtime image.
- Verify: deterministic output checksum; reverse-quantize roundtrip error
  bounded.

### Slice 3: Reference search
- API: `POST /fraud-score` uses brute-force scan over `i16` references with
  LLVM auto-vectorization (no manual SIMD intrinsics).
- No allocation per request on the hot path; mmap-friendly file format.
- Tests: known fixtures vs expected fraud decisions.
- Verify: smoke + bounded `k6 run test/test.js`; record p99/FP/FN/Err in
  vault `09-baseline-medicoes.md`.

### Slice 4: Submission topology
- DoD in **two tiers**:
  - **Functional gating (local).** `docker compose up --wait` on darwin/arm64
    (native, no QEMU); `k6 run test/smoke.js` passes; `k6 run test/test.js`
    runs to completion without compose crashing. Proves contract + absence of
    OOM/crash. NOT a performance signal.
  - **Measurement (closes the slice).** Open `rinha/test` issue against the
    official Rinha repo; Engine runs on Mac Mini Late 2014 (i5-4278U), posts
    `final_score`, p99, FP, FN, Err. Recorded in vault
    `09-baseline-medicoes.md`. No minimum score threshold; bad score becomes
    input for Slice 5+, does NOT reopen Slice 4.
- Pre-requisites (means, not DoD):
  - Multi-stage Dockerfile via `docker buildx --platform linux/amd64` with
    `cargo-chef` for dep cache. Preprocess runs in builder stage; runtime
    stage based on `gcr.io/distroless/cc-debian12:nonroot`. `data/*.bin`
    baked into the runtime image at `/data/` (no shared volume — see
    [ADR-0001](../docs/adr/0001-bake-data-in-image.md)).
  - `.cargo/config.toml` sets `target-cpu=x86-64-v3` for
    `x86_64-unknown-linux-gnu` only (see
    [ADR-0002](../docs/adr/0002-target-cpu-x86-64-v3.md)).
  - `[profile.release]`: `lto = "fat"`, `codegen-units = 1`,
    `panic = "abort"`, `strip = true`.
  - `--healthcheck` mode in the main binary using `std::net::TcpStream` + raw
    HTTP GET against `127.0.0.1:9999/ready`. Dockerfile `HEALTHCHECK` invokes
    it; compose `nginx.depends_on` uses `condition: service_healthy` on
    `api1`/`api2`.
  - Image pushed to GHCR as `v0.4.0` AFTER PR merge.
  - `submission` branch updated AFTER GHCR push:
    - `docker-compose.yml`: tag `v0.4.0`; budget split nginx `0.10 / 10MB`,
      api1/api2 `0.45 / 170MB` each; env vars per API:
      `TOKIO_WORKER_THREADS=1`, `MALLOC_ARENA_MAX=2`, `REFS_DATA_DIR=/data`.
  - `nginx.conf`: `worker_processes 1`, `events { use epoll;
      worker_connections 1024 }`, `access_log off`, `server_tokens off`,
      `upstream api { keepalive 32 }`.

### Slice 5: Exact indexed search
- Goal: escape the `final_score=-6000` floor without changing classifier
  output. Preserve exact equivalence with the brute-force reference:
  same canonical top-5, same `approved`, same `fraud_score`, tie-break by
  `(distance, original_index)`. Distance remains squared Euclidean over the
  current quantized `i16` vectors.
- First candidate: VP-Tree built at Docker image build time and baked into
  runtime (see [ADR-0003](../docs/adr/0003-bake-search-index-in-image.md)).
- Implementation bias: small in-repo VP-Tree implementation, not a generic
  dependency, so distance, tie-break, serialization, and hot path allocation
  stay under our control.
- First PR scope: VP-Tree only. Do not combine with `u8` quantization or
  manual SIMD; those become follow-up PRs measured against the VP-Tree result
  if needed.
- Follow-up order if exact VP-Tree still misses the failure-rate cut:
  manual SIMD over current `i16` distance first, `u8` quantization only after
  that because it reopens the sentinel/equivalence question.
- Runtime default: VP-Tree. Brute force stays available only as explicit
  validation mode (`SEARCH_MODE=bruteforce`); no silent fallback.
- Equivalence validation has two tiers: all `test/test-data.json` requests and
  a deterministic large sample generated from reference-derived payloads or
  query vectors.

### Slice 6: Robustness under host contention
- Goal: make the positive score the FLOOR, not the ceiling. A contended host
  (0.45 CPU disputed by a noisy neighbor) must still clear both cuts:
  failure < 15% AND p99 < 2000 ms. Exactness is already settled in Slice 5
  (0 FP/FN over the 3M dataset + differential fuzz); this slice is purely
  about latency tail, not classifier output.
- Root finding that motivates the slice: `fraud_score` handler calls
  `state.index.score(&q)` SYNCHRONOUSLY inside the async task, and
  `TOKIO_WORKER_THREADS=1` → one worker per API. The CPU-bound VP-Tree
  traversal blocks the executor; under contention queries serialize, the
  queue backs up, and p99 cascades into the k6 2001 ms timeout. Run 1 was
  fast only because a clean host hid the head-of-line blocking.
- **Phase 0 — reproduce locally (measurement-first).** Compose at 0.45 CPU
  per API + `stress-ng` noisy neighbor saturating the rest; run
  `k6 test.js`; confirm the p99 blowup reproduces off-Engine. This becomes
  the slice's test bench. No code changes until repro exists.
- **Phase 1 — exactness-preserving wins (no decision needed):**
  1. Move the CPU-bound search off the async executor (`spawn_blocking` or a
     dedicated pool sized to the CPU quota) so HTTP accept/enqueue is not
     held hostage by one slow query.
  2. Warm the page cache: `mlock`/pre-fault the `refs.i16.bin` +
     `vptree.nodes.bin` mmaps at startup so cold-page faults under memory
     pressure don't stall queries. Verify it fits the 350 MB total budget.
  3. Node layout: co-locate the VP point inside `VpNode` (today fetched via
     `ref_at` from a separate mmap → random access, cache miss per visit).
     Measure node visits/query first.
  4. Iterative search + explicit stack instead of recursion.
  5. Manual SIMD on the 14-dim i16 distance ONLY if profiling shows it pays.
- **Phase 2 — decision gate (needs owner approval; breaks "exact-first").**
  If Phase 1 doesn't tame the worst-case tail, switch to BOUNDED-work-per-query:
  ANN/HNSW with fixed `ef` (predictable per-query cost — the robustness lever
  VP-Tree backtracking lacks) and/or `u8` quantization (half the memory
  bandwidth). This reopens the locked Slice-5 exact-equivalence decision →
  architectural call, not executed without sign-off. Criterion would shift
  from byte-exact to recall ≥ target.
- **Phase 3 — re-measure and prove robustness.** New Engine prévia, capture
  artifacts, require MULTIPLE clean positive runs. One good run (#4880 run 1)
  does not prove robustness.
- GHCR `v0.6.0`; submission bump; tracked in GitHub issue (Slice 6).

## Deferred / Out of Scope

- `u8` quantization and ANN — only after exact VP-Tree is measured.
- Manual SIMD intrinsics (`std::arch::x86_64::_mm256_*`) — only if auto-vec
  proves insufficient after measurement.
- CI workflow for image build/push — manual pushes are fine until Slice 5+.
- Public PR to the official `participants/obrunogonzaga.json` — only after
  the first complete compose run.
- `/ready` validation of `metadata.json` (dims/scale literals) — gap inherited
  from Slice 3. Adds telemetry-style guards; Slice 5+.
- Shared-volume topology for kernel page-cache deduplication across API
  containers — revisit only if measurement shows per-container RSS at the
  170 MB limit (ADR-0001 supersession path).

## Open Questions

- Whether `u8` quantization fits with the `-1` sentinel without losing
  accuracy — investigate during Slice 5+.
