# Atomic Progress Log

Your temporal anchor. Tick atomic tasks as you complete them. Never mark a
task done unless `memory/verify.md` criteria are met.

The `state-enforcement.sh` hook blocks task completion if source files
changed but this file wasn't updated.

## In Progress

- [ ] Confirm first HTTP stack and implement Slice 1.

## Completed (this session)

- [x] Configure `agent-md.toml` — verified with `.agent-md/bin/doctor.sh`.
- [x] Enable Codex hooks in `~/.codex/config.toml` — verified `[features]`
  contains `codex_hooks = true`.
- [x] Enable git hook fallback — verified `core.hooksPath = .githooks` and
  `.agent-md/bin/doctor.sh`.
- [x] Seed agent-md memory files — updated project stack, plan, progress, and
  verification criteria from the local official docs; verified with
  `cargo check --locked`, `cargo fmt --check`,
  `cargo clippy --locked --all-targets --all-features -- -D warnings`, and
  `cargo test --locked`.

## Backlog (next up)

- [ ] Pick the minimal Rust HTTP stack for `GET /ready` and `POST /fraud-score`.
- [ ] Add request/response types and baseline endpoint tests.
- [ ] Run the service locally on port `9999` and pass `k6 run test/smoke.js`.
- [ ] Implement vectorization fixtures from `REGRAS_DE_DETECCAO.md`.

## Blocked

<!--
- [ ] <task> — waiting on: <reason or person>
-->
