# Modulo: Infraestrutura

## Estado

**Implementado** para a fonte bg3.wiki.

## Cliente HTTP

`MediaWikiClient` possui um `reqwest::Client` reutilizavel, URL-base fixa,
`User-Agent` configuravel e timeout global. A Action API recebe POST form-urlencoded;
a REST API recebe GET.

O cliente nao limita bytes nem trunca respostas. JSON e texto sao agregados em
memoria, conforme decisao explicita do projeto.

## Cache

`moka::future::Cache` armazena apenas respostas bem-sucedidas:

- TTL padrao: 300 segundos.
- Capacidade padrao: 512 entradas.
- Chave: operacao, URL e parametros normalizados.
- Erros, timeouts e respostas HTTP rejeitadas nao sao armazenados.

Uma entrada pode ter qualquer tamanho, portanto a capacidade por quantidade nao
representa limite previsivel de memoria.

## Concorrencia e retry

Um `tokio::Semaphore` limita chamadas externas; o default e uma requisicao por
vez. Operacoes de consulta sao idempotentes mesmo quando enviadas por POST.

Ha retry para `429`, `maxlag` e `5xx`. O cliente respeita `Retry-After` em
segundos e usa backoff exponencial com jitter. O default permite duas novas
tentativas. Timeout e resposta malformada sao normalizados sem expor detalhes.

## Rate limit MCP

O rate limit usa uma janela fixa de 60 segundos por IP observado no socket. O
default e 60 requisicoes. A tabela e limpa durante o uso para remover janelas
expiradas.

O servidor nao confia em headers de proxy. Atras de um reverse proxy, todos os
clientes podem compartilhar o mesmo bucket.

## Erros e logs

`WikiError` separa input invalido, not found, timeout, indisponibilidade,
rejeicao HTTP, erro da API e resposta inesperada. `public_message()` remove
status, codigos internos e causas.

Logs registram inicializacao e erros do servidor, nunca bodies integrais,
credenciais ou conteudo da wiki.

## Health check

`GET /health` devolve nome, versao e status do processo sem consultar a fonte.
Indisponibilidade da bg3.wiki nao torna o container unhealthy.
