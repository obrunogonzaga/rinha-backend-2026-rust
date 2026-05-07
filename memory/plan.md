# Macro Plan

The architectural design. Updated when direction changes. Vertical slices
only — build full-stack features end-to-end, not horizontal layers.

## Current Phase

Phase 1: Slice 1 — HTTP contract baseline.

## Closed Decisions

- HTTP stack: `axum` + `tokio`.
- Quantization: `i16` for reference vectors (`u8` reserved for Slice 5+).
- Preprocessing: build-time inside Docker multi-stage. No JSON parser at
  runtime, no startup decompression.
- Search baseline: brute-force SIMD. Upgrade path: VP-Tree → ANN (HNSW).
- Load balancer: nginx, round-robin only.
- Image registry: GHCR at `ghcr.io/obrunogonzaga/rinha-fraud-rust:vMAJOR.MINOR.PATCH`,
  one tag per slice release.
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
- API: `POST /fraud-score` uses brute-force SIMD over `i16` references.
- No allocation per request on the hot path; mmap-friendly file format.
- Tests: known fixtures vs expected fraud decisions.
- Verify: smoke + bounded `k6 run test/test.js`; record p99/FP/FN/Err in
  vault `09-baseline-medicoes.md`.

### Slice 4: Submission topology
- Multi-stage Dockerfile (preprocess → runtime). Image pushed to GHCR `:v0.1.0`.
- `docker-compose.yml` on `submission` branch: nginx + api1 + api2.
- Verify: compose up, `GET /ready` through LB, smoke through LB. Limits ≤ 1
  CPU and 350 MB total. amd64. Bridge network. Non-privileged.

## Deferred / Out of Scope

- `u8` quantization, VP-Tree, ANN — only after measured Slice 3 baseline.
- CI workflow for image build/push — manual pushes are fine until Slice 4.
- Public PR to the official `participants/obrunogonzaga.json` — only after
  the first complete compose run.

## Open Questions

- Whether mmap with shared file across both API containers reduces accounted
  RSS under cgroup v2 enough to matter — measure during Slice 4.
- Whether `u8` quantization fits with the `-1` sentinel without losing
  accuracy — investigate during Slice 5+.
