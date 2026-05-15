# Slice 4 — First official Engine measurement

**Date:** 2026-05-15T20:33:42Z
**Source:** [issue #4586](https://github.com/zanfranceschi/rinha-de-backend-2026/issues/4586) (prévia)
**Image:** `ghcr.io/obrunogonzaga/rinha-fraud-rust:v0.4.0` · submission `fb90b82`
**Hardware:** Mac Mini Late 2014, i5-4278U (Haswell 2c/4t), 8 GB, Ubuntu 24.04
**Limits:** official 1.0 CPU / 350 MB (nginx 0.10/10MB, api1/api2 0.45/170MB)

## Headline

`final_score = -6000` — the absolute floor. Both cuts triggered:

- **p99 cut**: p99 = 2002.11 ms ≥ 2000 ms ceiling → `p99_score = -3000`.
- **detection cut**: failure_rate = 89.13% ≥ 15% → `detection_score = -3000`.

## Breakdown

| Metric | Value |
|---|---|
| true_positive (fraud denied) | 707 |
| true_negative (legit approved) | 841 |
| false_positive | **0** |
| false_negative | **0** |
| http_errors | 12689 |
| completed (TP+TN+FP+FN) | 1548 |
| total attempted (incl. errors) | 14237 |
| weighted_errors_E | 63445 |

## Reading

1. **The classifier is correct.** Of every request that returned a 200,
   the approve/deny decision matched the expected label — 0 FP, 0 FN.
   The vectorization + i16 quantization + brute-force kNN pipeline is
   functionally right.
2. **The system is throughput-bound, not accuracy-bound.** 12689 of 14237
   attempts (89%) exceeded the k6 2001 ms timeout and were counted as
   HTTP errors. p99 pinned at ~2002 ms ≈ the timeout itself.
3. **Cause:** exact brute-force scans all 3,000,000 × 14 i16 values per
   request. Under 0.45 CPU per API on a 2014 Haswell, a single scan is
   far slower than the 1.1 ms/req implied by the 900 rps ramp target.
   Requests queue, latency explodes, k6 times them out.

## Comparison to the Slice 3 dev-box baseline

| | Slice 3 (M3, unconstrained) | Slice 4 (i5-4278U, 1 CPU) |
|---|---|---|
| p99 | 1257.34 ms | 2002.11 ms (capped) |
| FP / FN | 0 / 0 | 0 / 0 |
| http_errors | 0 | 12689 |
| final_score | 2900.55 | -6000 |

The algorithm did not change between slices; the topology and the
hardware did. The delta isolates exactly how much the exact brute-force
costs under the real constraint — which is the entire point of anchoring
the Slice 4 DoD on this measurement.

## Slice 5 targets (derived)

To escape the floor, in priority order:

1. **failure_rate < 15%** unlocks `detection_score` (currently -3000).
   This is the rigid cut — the single highest-value lever.
2. **p99 < 2000 ms** unlocks `p99_score`. Each 10× latency improvement is
   worth +1000 up to the +3000 ceiling at p99 ≤ 1 ms.

Candidate work (to be grilled before committing):

- **Approximate / structured search** — VP-Tree (exact, sublinear) or
  ANN (HNSW/IVF). Biggest expected lever; AVALIACAO.md explicitly hints
  brute force is too slow.
- **`u8` quantization** — halves memory bandwidth of the scan; pairs
  with the `-1` sentinel question already deferred in plan.md.
- **Manual SIMD** — `std::arch::x86_64::_mm256_*` for `sq_dist_14`
  instead of relying on LLVM auto-vectorization.

Deadline for the final test submission: **2026-06-05T23:59:59-03:00**.
