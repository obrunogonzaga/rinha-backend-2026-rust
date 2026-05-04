# SPECS - Rinha Backend 2026

## Stack inicial

- Linguagem: Rust.
- API HTTP: a definir, preferencia inicial por `axum` + `tokio`.
- Serializacao: `serde` / `serde_json`.
- Compressao gzip: crate a definir durante o preprocessador.
- Load balancer: `nginx` ou `haproxy`.
- Teste de carga: k6 usando scripts oficiais em `test/`.

## Estrutura esperada

```text
.
├── official/docs/br/        # copia local das docs oficiais
├── resources/               # arquivos oficiais de referencia
├── test/                    # scripts e massa de teste k6
├── src/                     # API Rust
├── data/                    # artefatos gerados, ignorados pelo git
├── PRD.md
├── SPECS.md
├── README.md
└── Cargo.toml
```

## Endpoints

### `GET /ready`

Retorna HTTP 2xx quando a API estiver pronta para receber trafego.

Resposta minima aceitavel:

```json
{
  "ready": true
}
```

### `POST /fraud-score`

Recebe uma transacao no formato oficial e retorna decisao.

Resposta:

```json
{
  "approved": true,
  "fraud_score": 0.0
}
```

Erros de parsing devem ser tratados com resposta HTTP apropriada durante desenvolvimento. Para ambiente competitivo, avaliar fallback que evite HTTP 5xx.

## Modelo de entrada

Campos obrigatorios:

- `id: string`
- `transaction.amount: number`
- `transaction.installments: integer`
- `transaction.requested_at: string ISO UTC`
- `customer.avg_amount: number`
- `customer.tx_count_24h: integer`
- `customer.known_merchants: string[]`
- `merchant.id: string`
- `merchant.mcc: string`
- `merchant.avg_amount: number`
- `terminal.is_online: boolean`
- `terminal.card_present: boolean`
- `terminal.km_from_home: number`
- `last_transaction: object | null`
- `last_transaction.timestamp: string ISO UTC`
- `last_transaction.km_from_current: number`

## Vetorizacao

Gerar vetor `[f32; 14]` nesta ordem:

| Indice | Dimensao | Regra |
| --- | --- | --- |
| 0 | `amount` | `clamp(transaction.amount / max_amount)` |
| 1 | `installments` | `clamp(transaction.installments / max_installments)` |
| 2 | `amount_vs_avg` | `clamp((transaction.amount / customer.avg_amount) / amount_vs_avg_ratio)` |
| 3 | `hour_of_day` | `hour_utc(transaction.requested_at) / 23` |
| 4 | `day_of_week` | `weekday_monday_zero(transaction.requested_at) / 6` |
| 5 | `minutes_since_last_tx` | `clamp(minutes / max_minutes)` ou `-1` |
| 6 | `km_from_last_tx` | `clamp(last_transaction.km_from_current / max_km)` ou `-1` |
| 7 | `km_from_home` | `clamp(terminal.km_from_home / max_km)` |
| 8 | `tx_count_24h` | `clamp(customer.tx_count_24h / max_tx_count_24h)` |
| 9 | `is_online` | `1` se true, senao `0` |
| 10 | `card_present` | `1` se true, senao `0` |
| 11 | `unknown_merchant` | `1` se `merchant.id` nao estiver em `known_merchants`, senao `0` |
| 12 | `mcc_risk` | valor de `mcc_risk.json`, default `0.5` |
| 13 | `merchant_avg_amount` | `clamp(merchant.avg_amount / max_merchant_avg_amount)` |

Constantes iniciais de `resources/normalization.json`:

```json
{
  "max_amount": 10000,
  "max_installments": 12,
  "amount_vs_avg_ratio": 10,
  "max_minutes": 1440,
  "max_km": 1000,
  "max_tx_count_24h": 20,
  "max_merchant_avg_amount": 10000
}
```

## Dataset

Fonte oficial:

- `resources/references.json.gz`
- `resources/mcc_risk.json`
- `resources/normalization.json`

Regra:

- Nao descompactar e versionar `references.json`.
- Criar preprocessador que leia `.gz` em streaming.
- Gerar artefatos em `data/`, ignorados pelo git.
- Preservar `-1` nos indices 5 e 6.

Formato binario candidato:

```text
data/references.f32.bin   # vetores contiguos: 3_000_000 * 14 * f32
data/labels.bin           # 1 byte por label, 1 = fraud, 0 = legit
data/metadata.json        # versao, contagem, dimensoes, estrategia
```

Alternativas futuras:

- quantizacao para `u16` ou `i16`;
- bitset para labels;
- particionamento por buckets;
- mmap para reduzir copia em memoria.

## Busca e decisao

Baseline:

- distancia euclidiana ao quadrado;
- top-5 menores distancias;
- sem `sqrt`, pois ordenacao nao precisa da raiz;
- sem alocacao por request no caminho quente.

Decisao:

```text
fraud_score = frauds_in_top_5 / 5
approved = fraud_score < 0.6
```

## Docker

Topologia minima:

```text
cliente -> load balancer :9999 -> api1
                              -> api2
```

Restricoes:

- total `cpus <= 1.0`;
- total `memory <= 350MB`;
- rede `bridge`;
- sem `privileged`;
- imagens publicas `linux-amd64`.

## Testes

Comandos alvo:

```bash
cargo check
cargo test
k6 run test/smoke.js
k6 run test/test.js
```

Testes unitarios obrigatorios:

- clamp;
- parsing de datas UTC;
- dia da semana com segunda = 0;
- `last_transaction: null`;
- MCC desconhecido;
- merchant conhecido/desconhecido;
- exemplos oficiais de vetorizacao.

## Politica de versionamento de dados

Versionar:

- docs oficiais raw;
- `resources/*.json`;
- `resources/references.json.gz`;
- scripts de teste oficiais;
- codigo-fonte;
- specs e notas.

Nao versionar:

- `data/`;
- `target/`;
- `results.json`;
- dataset descompactado;
- indices gerados.

