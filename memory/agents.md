# Sub-Agents & Tooling Registry

Update this file when sub-agents, MCPs, or core dependencies change.
Read at every session start.

## Active Sub-Agents

- None active. Use Codex/Claude directly unless a later slice needs an
  independent code review or performance investigation.

## MCPs / External Services

- Cargo toolchain — build, format, lint, and unit tests.
- k6 — official-style smoke/load testing against `http://localhost:9999`.
- agent-md hooks — repository-local verification contracts and memory checks.
- GitHub CLI (`gh`) — create repo, push, manage PRs.
- GHCR — `ghcr.io/obrunogonzaga/rinha-fraud-rust` (Docker image registry,
  populated from Slice 4 onward).

## Tech Stack

- Runtime: Rust binary service (`axum` + `tokio`).
- Language: Rust 2024 edition (toolchain 1.88+).
- Package manager/build: Cargo with `Cargo.lock`.
- HTTP: `axum`, `tokio` (`rt-multi-thread`, `macros`, `net` features).
- Serialization: `serde`, `serde_json`.
- API contract: `GET /ready` and `POST /fraud-score` on port `9999`.
- Reference data: `i16`-quantized binary `data/refs.i16.bin` (built at image
  build time from `resources/references.json.gz`).
- Search: brute-force SIMD (Slice 3); VP-Tree/ANN deferred.
- Load balancer: nginx, round-robin.
- Test assets: `test/smoke.js`, `test/test.js`, `test/test-data.json`.
- Verification config: `agent-md.toml`.

## Repository Topology

- Local development: `main` (source code) + feature branches.
- Public remote: `obrunogonzaga/rinha-backend-2026-rust` (GitHub).
- `submission` branch: orphan, deployment artifacts only
  (`docker-compose.yml`, `nginx.conf`, `info.json`). Source code forbidden.
- Image releases: per-slice semver tag on GHCR
  (`ghcr.io/obrunogonzaga/rinha-fraud-rust:v0.x.y`).
- License: MIT.

## Agent Runtime Policy

- Load only the relevant official doc sections for the current slice.
- Treat `AGENT.md` as the source of truth for agent behavior.
- Keep implementation passes bounded to one vertical slice.
- Reserve deeper reasoning for architecture, scoring quality, and performance
  decisions.

## Forbidden Patterns

- Do not use `test/test-data.json` as a fraud lookup table.
- Do not put business logic in the load balancer.
- Do not rely on HTTP errors for invalid runtime state; a fast valid response
  is usually cheaper than `5xx` in the scoring model.
- Do not add broad abstractions before measuring the simple path.
- Do not commit source code to the `submission` branch.
- Do not version `data/`, `target/`, or generated indices.
