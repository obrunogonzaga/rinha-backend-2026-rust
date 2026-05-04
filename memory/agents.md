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

## Tech Stack

- Runtime: Rust binary service.
- Language: Rust 2024 edition.
- Package manager/build: Cargo with `Cargo.lock`.
- API contract: `GET /ready` and `POST /fraud-score` on port `9999`.
- Test assets: `test/smoke.js`, `test/test.js`, and `test/test-data.json`.
- Reference assets: `resources/normalization.json`, `resources/mcc_risk.json`,
  and `resources/references.json.gz`.
- Official docs: `official/docs/br/`.
- Verification config: `agent-md.toml`.

## Agent Runtime Policy

- Load only the relevant official doc sections for the current slice.
- Treat `AGENT.md` as the source of truth for agent behavior.
- Keep implementation passes bounded to one vertical slice.
- Reserve deeper reasoning for architecture, scoring quality, and performance
  decisions.

## Forbidden Patterns

- Do not use `test/test-data.json` as a fraud lookup table.
- Do not put business logic in the load balancer.
- Do not rely on HTTP errors for invalid runtime state; a fast valid response is
  usually cheaper than `5xx` in the scoring model.
- Do not add broad abstractions before measuring the simple path.
