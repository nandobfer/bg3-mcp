# Diretrizes do Projeto

## Objetivo

Manter um servidor MCP publico e comunitario, escrito em Rust, para consultas sob
demanda de informacoes de Baldur's Gate 3. A fonte implementada e a bg3.wiki. O
dominio de mods so deve ser iniciado depois da escolha de um provedor.

## Estado implementado

```text
MCP client
    |
    v
Axum / Streamable HTTP
    |
    v
rmcp WikiMcpServer --> WikiService --> MediaWikiClient --> bg3.wiki
                          |                  |
                          v                  v
                     transformacao    cache, timeout,
                     e atribuicao     concorrencia, retry
```

O servidor usa `rmcp 3.1.2`, Axum e Streamable HTTP stateless. A aplicacao expoe
`/mcp` e `/health`. Nao ha transporte `stdio`.

## Estrutura

```text
src/
  main.rs
  lib.rs
  config.rs
  error.rs
  server.rs
  mcp.rs
  wiki/
    mod.rs
    client.rs
    models.rs
    service.rs
  infrastructure/
    mod.rs
    http.rs
    cache.rs
tests/
  common/
  http_server.rs
  wiki_service.rs
```

`main.rs` apenas carrega ambiente, inicia tracing e chama o servidor. Handlers MCP
delegam ao `WikiService`; requisicoes MediaWiki pertencem ao cliente HTTP.

## Dependencias principais

- `rmcp`: protocolo, macros de ferramentas e Streamable HTTP.
- `axum` e `tower-http`: servidor, rotas e CORS.
- `tokio`: runtime e cancelamento.
- `reqwest`: Action API e REST sobre rustls.
- `serde`, `serde_json` e `schemars`: contratos e schemas.
- `moka`: cache em memoria.
- `scraper` e `ammonia`: fragmentos, texto e HTML sanitizado.
- `tracing`: logs estruturados.
- `thiserror`: erros tipados.

Rust esta fixado em `1.89.0`; versoes transitivas ficam em `Cargo.lock`.

## Configuracao

| Variavel | Default | Finalidade |
| --- | --- | --- |
| `BG3_WIKI_BASE_URL` | `https://bg3.wiki` | Origem MediaWiki ou mock |
| `BG3_MCP_USER_AGENT` | obrigatoria | Identificacao da integracao |
| `BG3_MCP_HTTP_TIMEOUT_SECS` | `15` | Timeout externo |
| `BG3_MCP_MAX_CONCURRENCY` | `1` | Chamadas simultaneas a fonte |
| `BG3_MCP_CACHE_TTL_SECS` | `300` | TTL do cache HTTP |
| `BG3_MCP_CACHE_MAX_ENTRIES` | `512` | Quantidade de respostas em cache |
| `BG3_MCP_HTTP_RETRY_MAX` | `2` | Retries apos a primeira tentativa |
| `BG3_MCP_RATE_LIMIT_PER_MINUTE` | `60` | Requisicoes MCP por IP |
| `BG3_MCP_LOG` | `info` | Filtro do tracing |
| `BG3_MCP_HOST` | `0.0.0.0` | Interface HTTP |
| `BG3_MCP_PORT` | `3000` | Porta HTTP |
| `BG3_MCP_TRANSPORT` | `streamable-http` | Transporte aceito |

## Decisoes publicas

- Servico comunitario, publico e sem autenticacao.
- CORS permissivo.
- Validacao de `Host` e `Origin` desativada no `rmcp`.
- Nenhum limite de bytes imposto a bodies MCP ou respostas externas.
- Nenhum truncamento de paginas e secoes.
- Rate limit por IP e limite de quantidade para pesquisa e links permanecem.
- Conteudo e retornado no idioma original, sem traducao.

Essas escolhas ampliam risco de abuso e consumo de memoria e divergem das
recomendacoes de seguranca do transporte MCP. Nao as altere silenciosamente.

## Limites de dominio

- Consulta somente sob demanda; crawling e espelhamento sao proibidos.
- Wiki e mods permanecem dominios separados.
- Respostas externas sempre incluem atribuicao e URL.
- Testes automatizados sempre usam fonte mockada.
- `/health` verifica somente o processo.

## Pendencias

- Substituir `CHANGE_ME` no `User-Agent` distribuido como exemplo.
- Confirmar volume publico aceitavel com os mantenedores da bg3.wiki.
- Definir reverse proxy, TLS, registry e hospedagem de producao.
- Escolher e validar o provedor de mods.
