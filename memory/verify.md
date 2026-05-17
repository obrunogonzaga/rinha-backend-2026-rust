# Definition of Done

Every task's verification criteria must pass before it is marked complete
in `progress.md`. No exceptions.

## Text Verification (always required)

- [ ] `cargo check --locked`
- [ ] `cargo fmt --check`
- [ ] `cargo clippy --locked --all-targets --all-features -- -D warnings`
- [ ] `cargo test --locked`

## Tactile Verification (when code executes)

- [ ] Code was actually run — not just written. Script ran, endpoint
  responded, CLI output observed.
- [ ] Logs checked — no unexpected errors, warnings, or deprecations.
- [ ] At least one happy path and one edge case exercised manually.

## Visual Verification (UI changes only)

- [ ] Screenshot captured via Playwright (`.agent-md/bin/playwright-capture.sh`)
- [ ] VLM or human review confirms visual intent matches the spec
- [ ] No self-grading ("the code looks right") — independent verification

## Independent Verification

- [ ] Not self-graded. One of: sub-agent review, test suite, or the human
  confirmed.

## Structured Output / Tool Verification

- [ ] Tool arguments and structured outputs were validated before use
  (required fields, types, enum values, and file paths).
- [ ] Tool failures used structured error information where available:
  `status`, `type`, `message`, `suggestion`.
- [ ] High-risk claims or changes had an adversarial or independent check.

## Task-Specific Criteria

### Slice 1: HTTP contract baseline
- [ ] Service listens on `localhost:9999`.
- [ ] `GET /ready` returns `HTTP 2xx`.
- [ ] `POST /fraud-score` returns `HTTP 200` with JSON fields
  `approved: boolean` and `fraud_score: number`.
- [ ] Invalid or degraded `POST /fraud-score` paths return `HTTP 200` with
  fallback `approved: true` and `fraud_score: 0.0`, not `4xx`/`5xx`.
- [ ] `k6 run test/smoke.js` passes while the service is running.

### Slice 2: Vectorization and deterministic scoring fixtures
- [ ] The 14 vector dimensions match the examples in
  `official/docs/br/REGRAS_DE_DETECCAO.md`.
- [ ] `last_transaction: null` maps dimensions 5 and 6 to `-1`.
- [ ] `approved` uses `fraud_score < 0.6`.

### Slice 3: Reference search
- [ ] `resources/references.json.gz` is loaded or preprocessed without using
  `test/test-data.json` as a lookup source.
- [ ] Docker build path generates binary reference artifacts before runtime;
  runtime maps read-only artifacts and validates metadata before `/ready`.
- [ ] Microbench covers vectorization and top-k/kNN isolated from HTTP.
- [ ] If brute-force baseline p99 is above `50ms` under controlled local load,
  progress records an ANN/structured-index or quantization task before further
  HTTP micro-optimization.
- [ ] A bounded `k6 run test/test.js` or documented smaller equivalent produces
  `test/results.json`.
- [ ] Result inspection includes p99, HTTP errors, FP, FN, and final score.

### Slice 4: Submission topology

Structural (pre-requisites):
- [ ] `docker-compose.yml` exposes only the load balancer on port `9999`.
- [ ] At least two API instances receive traffic through round-robin.
- [ ] Total declared limits sum to exactly `1.0 CPU` and `350 MB`
  (nginx `0.10 / 10MB`, api1/api2 `0.45 / 170MB` each).
- [ ] `nginx.depends_on` uses `condition: service_healthy` for both APIs.
- [ ] Image is `linux/amd64`, public on GHCR as `v0.4.0`.
- [ ] `submission` branch carries only `docker-compose.yml`, `nginx.conf`,
  `info.json`, `README.md` — no source code.

Tier 1 — Functional gating (local, darwin/arm64):
- [ ] `docker compose up --wait` returns successfully (i.e., healthchecks
  pass within `start-period`).
- [ ] `k6 run test/smoke.js` passes with `http_req_failed: rate==0.0` and
  `checks: rate==1.0`.
- [ ] `k6 run test/test.js` runs to completion without a container crashing
  or compose returning a non-zero exit. RSS per container observed below the
  configured limit.

Tier 2 — Measurement (closes the slice):
- [ ] Either: official Engine result obtained via `rinha/test` issue and
  recorded in vault `09-baseline-medicoes.md` with `final_score`, p99, FP,
  FN, Err.
- [ ] Or fallback: linux/amd64 VPS run, recorded with explicit "proxy
  measurement" annotation in the same vault note.
- [ ] No minimum score gate. A poor score (cuts triggered) is a valid
  outcome — it scopes Slice 5+ work but does NOT reopen Slice 4.

### Slice 5: Exact indexed search
- [x] Optimized search returns the same `approved` and `fraud_score` as
  brute force for every request in `test/test-data.json`. (54,100 entries,
  0 mismatch — `validate_equivalence`.)
- [x] Optimized search returns the same `approved` and `fraud_score` as
  brute force for a deterministic large sample generated from
  reference-derived payloads or query vectors. (Fixed-seed SplitMix64 fuzz:
  reference-derived exact/near/far-perturbed + uniform-random; 5000 queries
  on the full 3,000,000-ref dataset, 0 mismatch.)
- [x] Internal search tests compare canonical top-5 neighbors, not only API
  output. (`validate_equivalence` top-5 sample + random fuzz; unit tests.)
- [x] Distance ties use canonical order: `(distance, original_index)`.
  (Shared `TopK`/`Neighbor` ordering; unit tests for ties.)
- [x] VP-Tree index is generated at Docker build time and loaded from baked
  runtime artifacts. (preprocess writes `vptree.*`; Dockerfile bakes `/data`.)
- [x] Brute force remains available only through explicit validation mode
  (`SEARCH_MODE=bruteforce`); optimized mode must not silently fall back.
  (Tests assert load fails on missing/corrupt artifacts with no fallback.)
- [x] Result inspection includes p99, HTTP errors, FP, FN, failure_rate, and
  final_score versus the Slice 4 official baseline.
  (`bench/slice-5/.../ANALYSIS.md`.)
- [x] Pruning is exact without floating point: VP-Tree triangle-inequality
  bounds evaluated in integer space (`sqrt_le_sum`, no `f64::sqrt` on the
  search path), so exactness holds for unseen Engine queries, not only
  fixtures.
- [ ] Publish `v0.5.0` for Engine only after local official-style
  `k6 run test/test.js` shows no equivalence regressions (0 FP / 0 FN against
  expected labels) and a large HTTP-error reduction versus Slice 4 baseline.
  Local p99 is a sanity signal, not the final judge.
