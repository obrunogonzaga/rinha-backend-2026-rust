# Rinha Backend 2026

Implementacao em Rust para a Rinha de Backend 2026.

## Estrutura inicial

- `official/docs/br/`: copia local dos documentos oficiais usados como referencia.
- `resources/`: arquivos oficiais de referencia e exemplos.
- `test/`: scripts e massa de teste oficiais.
- `.claude/napkin.md`: runbook local para orientar sessoes futuras.

## Decisao de repositorio

Este projeto parte do zero. O repositorio oficial da Rinha fica apenas como fonte de documentacao, recursos e scripts de teste.

## Decisoes V1

- Stack HTTP inicial: `axum` + `tokio`, com reavaliacao para `hyper` direto
  apenas se medicao mostrar custo relevante.
- Dataset: `references.json.gz` deve ser preprocessado no build da imagem para
  artefatos binarios e carregado no runtime por `mmap` read-only.
- Erros no caminho competitivo: `POST /fraud-score` deve preferir `HTTP 200`
  com fallback `{"approved": true, "fraud_score": 0.0}` em vez de propagar
  `5xx`.
- Memoria: o limite de `350MB` precisa ser verificado por uso real sob carga
  com duas APIs e load balancer, nao apenas por configuracao declarada.
