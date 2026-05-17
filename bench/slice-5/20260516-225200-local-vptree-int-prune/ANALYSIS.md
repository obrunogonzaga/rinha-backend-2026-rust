# Slice 5 — Local VP-Tree measurement (integer-exact pruning)

**Date:** 2026-05-16
**Commit:** `2bba584` (`feat/slice-5-vptree`, PR #12)
**Source:** local darwin/arm64 release binary, single API, no nginx/cgroup
**Baseline:** Slice 4 official Engine `v0.4.0`, submission `fb90b82`
**Prior local run:** `bench/slice-5/20260516-121352-local-vptree-u64-node20/`

## Result

`results.json` in this directory is the verbatim k6 `handleSummary` output
of the re-observed run (p99 9.81ms).

| Metric | Slice 4 Engine | Slice 5 local (f64 prune) | Slice 5 local (int prune) |
|---|---:|---:|---:|
| p99 | 2002.11ms | 3.16ms | 9.81ms |
| failure_rate | 89.13% | 0% | 0% |
| FP / FN | 0 / 0 | 0 / 0 | 0 / 0 |
| http_errors | 12689 | 0 | 0 |
| final_score | -6000 | 5500.89 | 5008.53 |
| iterations | n/a | ~54k | 54059 / 54100 |

## Reading

This run validates the review follow-up: VP-Tree pruning moved from
`f64::sqrt` + `f64::EPSILON` padding to integer-exact `sqrt_le_sum`
(`i128`/`u128`, no float on the search path). The decisive signals —
**FP=0, FN=0, http_errors=0, failure_rate=0%** — confirm the classifier
output is preserved exactly. Both score cuts stay clear (p99 9.81ms ≪
2000ms; failure_rate 0% ≪ 15%).

Local p99 is noise-sensitive on a shared dev box: an earlier background
run of this same commit produced p99 31.91ms, this quieter run produced
9.81ms. The number is a sanity signal, not the judge — the authoritative
measurement is the official Engine. There is no pruning regression: VP-Tree
construction and node serialization are byte-identical between the two
commits (verified — the artifact checksum is `635246948b295449` both
before and after; the diff only touches `search_node`, adds `sqrt_le_sum`,
and updates the validator/docs), and integer pruning is more
conservative-exact than the epsilon-padded float version.

## Equivalence evidence

`validate_equivalence data test/test-data.json 512 5000` on the full
3,000,000-reference dataset → 0 mismatches: 54,100 official expected
outputs + 512 strided canonical top-5 + 5000 deterministic differential
queries (reference-derived exact/near/far-perturbed + uniform-random).
`cargo fmt --check` / `clippy -D warnings` clean, 51 unit tests pass.
k6 `smoke.js` 20/20 checks succeeded, 0 http failures.

## Artifacts

- `results.json` — k6 scoring output (verbatim, this re-observed run)
- `metadata.json` — reference artifact metadata
- `vptree.metadata.json` — VP-Tree artifact metadata and checksum
  (`635246948b295449`, regenerated and verified for this build)
