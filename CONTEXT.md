# Rinha Backend 2026

Glossario do projeto para separar linguagem de engenharia, avaliacao e serie editorial.

## Language

**Slice**:
Unidade incremental de engenharia entregue e medida no backend.
_Avoid_: post, artigo

**Post**:
Entrada sequencial da serie editorial do blog, identificada por `series.order`.
_Avoid_: slice, slot do brainstorm

**Medicao oficial**:
Resultado produzido pela Engine da Rinha contra a branch de submissao.
_Avoid_: benchmark local, smoke test

**Failure rate**:
Percentual bruto de respostas incorretas ou erros HTTP no teste de avaliacao.
_Avoid_: taxa ponderada, error_rate_epsilon

**Equivalencia exata**:
Garantia de que uma otimizacao preserva o mesmo top-5 canonico e, por consequencia, o mesmo `approved` e `fraud_score` do brute force de referencia.
_Avoid_: ANN, aproximado

**Desempate canonico**:
Ordem por menor distancia e, em empate, menor indice original no dataset de referencias.
_Avoid_: ordem do indice, ordem arbitraria

**Distancia canonica**:
Distancia euclidiana ao quadrado calculada sobre vetores quantizados `i16`.
_Avoid_: distancia f32, metrica alternativa

## Relationships

- Uma **Slice** pode alimentar um ou mais **Posts**.
- Um **Post** pode documentar uma **Slice** anterior depois de uma **Medicao oficial** posterior.
- Uma **Medicao oficial** produz a **Failure rate** que decide o corte de deteccao.
- **Equivalencia exata** limita a Slice 5 a otimizacoes que nao trocam acuracia por throughput.
- **Desempate canonico** define quando uma busca preserva **Equivalencia exata**.
- **Distancia canonica** alimenta o **Desempate canonico** e o top-5.

## Example dialogue

> **Dev:** "O post 5 fala Slice 3 mas usa o -6000 da medicao oficial."
> **Domain expert:** "Correto: a Slice 3 criou o brute force; a Slice 4 produziu a medicao oficial que fechou o post."

## Flagged ambiguities

- "Slice 4 -> post 8" confundiu slot de brainstorm editorial com `series.order`; resolvido: publicacao segue **Post** sequencial.
- "Slice 5" significa proxima unidade de engenharia, nao o post publicado mais recente.
- "Otimizar throughput" poderia incluir ANN; resolvido para o inicio da Slice 5: exigir **Equivalencia exata**.
- "Mesmos vizinhos" inclui desempate por ordem original do dataset, nao por ordem interna do indice.
- "Busca exata" significa exata contra a **Distancia canonica** atual, nao contra os valores `f32` originais.
