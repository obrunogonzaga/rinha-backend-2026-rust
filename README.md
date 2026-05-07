# rinha-backend-2026-rust — submission branch

Deployment-only artifacts. No source code, by spec
([SUBMISSAO.md](https://github.com/zanfranceschi/rinha-de-backend-2026/blob/main/docs/br/SUBMISSAO.md)).

- Source code: <https://github.com/obrunogonzaga/rinha-backend-2026-rust> (`main`).
- Image: `ghcr.io/obrunogonzaga/rinha-fraud-rust:vMAJOR.MINOR.PATCH`, one tag per slice release.

## Run

```bash
docker compose up
```

Load balancer listens on `:9999` and round-robins between `api1` and `api2`.

## Notes

The image tag pinned in `docker-compose.yml` may not exist yet at every commit on this branch — the source repo updates this branch alongside each released image tag.
