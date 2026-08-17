# Modulo: Infraestrutura

## Objetivo

Fornecer comportamento operacional compartilhado sem acoplar os dominios de
wiki e mods.

## Cliente HTTP

- Um cliente reutilizavel por processo.
- URL-base configuravel por fonte.
- `User-Agent` configuravel.
- Timeout por requisicao.
- Limite de tamanho de resposta.
- Redirects restritos e observaveis.

## Concorrencia

Cada fonte deve possuir limite de concorrencia. A configuracao inicial usa
`BG3_MCP_MAX_CONCURRENCY`, mas a implementacao pode evoluir para limites por
fonte quando houver mais de uma integracao real.

## Cache

O cache inicial deve ser em memoria, salvo se o perfil de uso demonstrar a
necessidade de persistencia. Chaves precisam considerar operacao, parametros
normalizados e fonte. Nao armazene erros transitorios como sucesso.

Politica final de tamanho, eviction e cache negativo: **TBD**.

## Retry

- Apenas operacoes idempotentes.
- Backoff exponencial limitado.
- Respeitar `Retry-After`.
- Retry para `429`, timeout de conexao e falhas `5xx` selecionadas.
- Sem retry para input invalido, autenticacao, permissao ou not found.

## Erros

Erros internos devem preservar causa para logs e observabilidade, enquanto o
contrato MCP recebe uma categoria segura e acionavel. Corpos externos e stack
traces nao devem aparecer para o cliente.

## Tracing

Registre operacao, fonte, status, latencia, cache hit/miss e categoria de erro.
Nao registre credenciais, headers de autorizacao ou conteudo integral retornado
pelas fontes.

## Health check

`GET /health` deve verificar apenas a saude do processo. Nao consulte a
bg3.wiki, pois indisponibilidade externa nao deve reiniciar continuamente um
processo saudavel.
