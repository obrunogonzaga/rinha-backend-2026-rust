# Macro Plan

The architectural design. Updated when direction changes. Vertical slices
only — build full-stack features end-to-end, not horizontal layers
(all DBs, then all APIs, then all UIs).

## Current Phase

Phase 0: repository setup and implementation plan.

## Vertical Slices

### Slice 1: HTTP contract baseline
- **API**: expose `GET /ready` and `POST /fraud-score` on port `9999`.
- **Data**: parse the request shape from `official/docs/br/API.md`.
- **Decision**: return a deterministic placeholder only until vector scoring is
  implemented.
- **Tests**: unit tests for request/response shape and `k6 run test/smoke.js`
  against the running service.
- **Verify**: universal Cargo checks plus the smoke test in `memory/verify.md`.

### Slice 2: Vectorization and deterministic scoring fixtures
- **API**: keep the same endpoint contract.
- **Data**: load `resources/normalization.json` and `resources/mcc_risk.json`.
- **Decision**: implement the 14-dimension vector from
  `official/docs/br/REGRAS_DE_DETECCAO.md`.
- **Tests**: fixture tests for legitimate and fraudulent examples from the docs,
  including `last_transaction: null`.
- **Verify**: unit tests prove vector values and response threshold behavior.

### Slice 3: Reference search
- **API**: `POST /fraud-score` uses nearest references instead of placeholders.
- **Data**: load/preprocess `resources/references.json.gz` without using
  `test/test-data.json` as lookup data.
- **Decision**: start with the simplest correct search, then optimize based on
  measured p99 and memory use.
- **Tests**: compare known fixtures against expected fraud decisions.
- **Verify**: smoke test plus a bounded run of the official k6 script.

### Slice 4: Submission topology
- **API**: two API instances behind a load balancer on host port `9999`.
- **Data**: reference data available inside the container image/runtime.
- **Decision**: keep the load balancer as round-robin only.
- **Tests**: compose startup, `GET /ready`, smoke test through the load balancer.
- **Verify**: Docker Compose respects `bridge`, no `privileged`, `linux-amd64`,
  and total declared limits no higher than `1 CPU` and `350 MB`.

## Deferred / Out of Scope

- ANN/vector index optimization until the baseline is correct and measured.
- CI publishing, remote pushes, or branch `submission` work until requested.
- UI or visual verification; this project is backend-only.

## Open Questions

- Which Rust HTTP stack to use for the first implementation slice.
- Whether startup preprocessing may generate local artifacts or must happen
  entirely at runtime/container build time.
- Target strategy for memory allocation across load balancer and two APIs under
  the `350 MB` total limit.
