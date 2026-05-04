# PRD - Rinha Backend 2026

## Visao geral

Construir uma API de deteccao de fraude para a Rinha de Backend 2026 usando Rust. A API recebe transacoes de cartao, transforma o payload em um vetor de 14 dimensoes, consulta referencias rotuladas e retorna uma decisao de aprovacao com score de fraude.

Este projeto tambem deve servir como base para uma serie de artigos em blog pessoal e LinkedIn documentando a jornada tecnica.

## Objetivos

- Implementar uma solucao funcional e submetivel para a Rinha de Backend 2026.
- Usar Rust como linguagem principal.
- Manter compatibilidade com as regras oficiais de API, arquitetura, dataset e avaliacao.
- Otimizar progressivamente latencia, memoria e qualidade de deteccao.
- Registrar decisoes, metricas e aprendizados para conteudo editorial.

## Fora de escopo

- Implementar o sistema de autorizacao de cartao.
- Usar payloads do teste como lookup de fraude.
- Criar logica de negocio no load balancer.
- Depender de infraestrutura fora do `docker-compose.yml`.
- Versionar artefatos gerados pesados, como dataset descompactado ou indices binarios.

## Usuarios

- Avaliador automatico da Rinha, que executa testes contra `localhost:9999`.
- Desenvolvedor do projeto, que precisa iterar com benchmark e validar tradeoffs.
- Leitores dos artigos, que acompanham decisoes tecnicas, erros e resultados.

## Requisitos funcionais

- Expor `GET /ready`.
- Expor `POST /fraud-score`.
- Aceitar o payload oficial de transacao.
- Retornar JSON no formato:

```json
{
  "approved": false,
  "fraud_score": 1.0
}
```

- Gerar vetor de 14 dimensoes seguindo as regras oficiais.
- Buscar os 5 vizinhos mais proximos ou usar tecnica equivalente que minimize erro.
- Calcular `fraud_score = fraudes_entre_5 / 5`.
- Calcular `approved = fraud_score < 0.6`.

## Requisitos nao funcionais

- Responder na porta `9999` via load balancer.
- Ter pelo menos 2 instancias da API.
- Usar rede Docker `bridge`.
- Nao usar `privileged`.
- Limitar todos os servicos a no maximo `1 CPU` e `350 MB`.
- Usar imagens publicas compativeis com `linux-amd64`.
- Priorizar evitar HTTP errors, pois eles pesam mais que erro de classificacao.
- Manter startup previsivel e observavel.

## Metricas de sucesso

- `k6 run test/smoke.js` passa sem falhas.
- `k6 run test/test.js` executa sem HTTP errors relevantes.
- Taxa de falhas `(FP + FN + Err) / N` fica bem abaixo de 15%.
- p99 cai progressivamente a cada iteracao.
- Uso de memoria cabe no limite com duas APIs e load balancer.
- Decisoes e metricas importantes sao registradas no vault.

## Riscos

- `references.json.gz` descompacta para centenas de MB; carregar JSON em memoria estoura o limite.
- Busca brute force exata em 3 milhoes de vetores pode ter p99 alto.
- Otimizacoes aproximadas podem aumentar FP/FN.
- Duas instancias de API duplicam memoria se cada uma carregar todo o indice.
- Imagem construida em Mac pode sair apenas `arm64` se nao houver cuidado.

## Marcos

1. API minima em Rust com smoke test.
2. Vetorizacao oficial com testes unitarios.
3. Preprocessador lendo `references.json.gz` em streaming.
4. Baseline de busca exata e medicao.
5. Formato binario compacto.
6. Docker Compose com load balancer e duas APIs.
7. Otimizacoes de busca e memoria.
8. Branch `submission` com artefatos finais.

