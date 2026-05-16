# Slice 5 — Local VP-Tree measurement

**Date:** 2026-05-16T12:13:52Z
**Source:** local darwin/arm64 release binary, single API, no nginx/cgroup
**Baseline:** Slice 4 official Engine `v0.4.0`, submission `fb90b82`

## Result

| Metric | Slice 4 Engine | Slice 5 local VP-Tree |
|---|---:|---:|
| p99 | 2002.11ms | 3.16ms |
| failure_rate | 89.13% | 0% |
| FP / FN | 0 / 0 | 0 / 0 |
| http_errors | 12689 | 0 |
| final_score | -6000 | 5500.89 |
| process RSS after load | n/a | 146.8 MiB |

## Reading

The VP-Tree preserves classifier output and removes local timeouts under the
official-style k6 ramp. This is not an official score because it ran on the
dev machine without the submission topology, but it clears the local publish
gate for `v0.5.0`: no FP/FN regression and a large HTTP-error reduction.

Distances and VP-Tree thresholds are stored as `u64`, preserving the full
theoretical squared-distance range for 14 `i16` dimensions. Nodes serialize to
20 bytes (`u32 point_index`, `u64 threshold_sq`, `u32 left`, `u32 right`) to
keep the runtime mmap footprint below the per-API 170 MiB budget.

## Artifacts

- `results.json` — k6 scoring output
- `metadata.json` — reference artifact metadata
- `vptree.metadata.json` — VP-Tree artifact metadata and checksum
