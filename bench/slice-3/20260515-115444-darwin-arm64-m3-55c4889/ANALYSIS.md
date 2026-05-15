# Slice 3 dev-box baseline — análise

Companheiro de `metadata.json` + `results.json`. Lê o `headline` lá
primeiro, volta aqui pra contexto.

## Pergunta: dropped iterations (10.961) preocupam?

**Resposta curta:** estruturalmente sim, no score deste run não.

### Por que não conta no score

A regra oficial em `official/docs/br/AVALIACAO.md` define
`N = TP + TN + FP + FN + Err`, onde `Err` é "resposta HTTP diferente de
200". Dropped iteration é o k6 desistindo de disparar antes de abrir o
socket — nunca vira request, não vira nenhum dos cinco buckets. Logo,
`failure_rate` e `weighted_errors_E` ignoram. Os 43.098 que entraram
foram avaliados; os 10.961 sumiram silenciosos.

### Por que ainda incomoda

1. **Sinal de saturação.** Rampa pediu até 900 req/s; throughput médio
   sustentado foi 357 req/s. Drop só acontece porque os 250 VUs do pool
   ficaram presos em flight aguardando o servidor. É o p99 1257 ms se
   manifestando como starvation no gerador de carga, não no servidor.

2. **Slice 4 vai piorar.** Hoje o processo usa os 8 cores do M3 livres.
   No compose oficial — 1 CPU total dividida entre nginx + api1 + api2 —
   o throughput-teto cai junto com o p99. Mais drops, mesma fórmula.

3. **O script oficial pode contar diferente.** `AVALIACAO.md` avisa:
   *"O script disponibilizado aqui serve para você rodar os seus
   próprios testes e pode não ser idêntico à versão final usada na
   avaliação oficial."* Versão final pode usar `shared-iterations`
   (não dropa, estende a duração), `constant-arrival-rate` com VU cap
   maior, ou marcar drops como `Err`. Todos esses caminhos punem.

### Implicação operacional

Drops são sintoma de p99 alto, não métrica independente. Otimizar pra
"menos drops" antes de baixar p99 é tratar o termômetro. Quando o p99
cair pra dezena de ms (ANN/quantização u8/SIMD em Slice 5+), os 250 VUs
liberam rápido o suficiente pra absorver os 900 req/s da rampa e o
contador zera sozinho.

**Não usar este score como prova de viabilidade.** É um teto otimista —
fórmula mediu só o que o servidor viu. Real run com Linux 1 CPU verá
mais requests por VU livre e potencialmente mais erros absolutos.

## Pontos de contraste pro Slice 4

Quando rodar o k6 contra o compose, comparar:

| Métrica           | Slice 3 dev-box | Slice 4 compose (esperado) |
|-------------------|-----------------|----------------------------|
| p99               | 1257 ms         | provavelmente pior (1 CPU) |
| iterações ok      | 43.098          | ?                          |
| dropped           | 10.961          | provavelmente maior        |
| FP / FN / Err     | 0 / 0 / 0       | manter zero é a meta       |
| RSS por container | n/a (1 proc 85MB) | api1+api2+nginx ≤ 350MB    |
| `final_score`     | 2900.55         | ?                          |

O delta entre "perfeição algorítmica" (Slice 3, sem constraint) e
"realidade competitiva" (Slice 4, 1 CPU) isola o custo do constraint.
Slice 5+ deve recuperar e ultrapassar.

## Referências

- `official/docs/br/AVALIACAO.md` — fórmula `score_p99 + score_det`,
  buckets, cortes 15% e 2000ms.
- `memory/verify.md` — Slice 3 DoD: "p99 above 50ms ... progress
  records an ANN/structured-index or quantization task". Trigger hit.
- `memory/plan.md` — Slice 5+ upgrade path: VP-Tree → HNSW; u8
  quantization.
