# Modulo: Infraestrutura

## Estado

**Implementado** para bg3.wiki e mod.io.

## Cliente HTTP

`MediaWikiClient` possui um `reqwest::Client` reutilizavel, URL-base fixa,
`User-Agent` configuravel e timeout global. A Action API recebe POST form-urlencoded;
a REST API recebe GET.

O cliente nao limita bytes nem trunca respostas. JSON e texto sao agregados em
memoria, conforme decisao explicita do projeto.

`ModIoClient` usa GET sobre a REST API v1, envia a API key no query parameter
exigido pelo provedor e `X-Modio-Platform` em cada chamada. Redirects sao
desativados para que uma credencial presente na URL nunca seja encaminhada para
outra origem. Cache keys e logs nunca incluem a API key.

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

O mod.io possui cache, semaforo e retry proprios. Uma janela global limita o uso
da chave ao default de 60 requisicoes por minuto. Ao receber `429`, um cooldown
compartilhado impede novas chamadas ate `Retry-After`; na ausencia do header, o
fallback e 60 segundos. Valores maiores tambem sao limitados a 60 segundos para
manter o backoff finito.

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

`ModIoError` tambem distingue credencial rejeitada. Nenhum erro inclui API key,
query string, body externo ou URL interna de mock.

Logs registram inicializacao e erros do servidor, nunca bodies integrais,
credenciais ou conteudo da wiki.

## Health check

`GET /health` devolve nome, versao e status do processo sem consultar a fonte.
Indisponibilidade da bg3.wiki nao torna o container unhealthy.
