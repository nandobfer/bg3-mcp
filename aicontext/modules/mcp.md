# Modulo: MCP

## Estado

**Implementado** com `rmcp 3.1.2` e Streamable HTTP stateless.

## Endpoints

| Metodo | Caminho | Uso |
| --- | --- | --- |
| `GET` | `/health` | Saude local do processo |
| `POST`, `GET`, `DELETE` | `/mcp` | Transporte MCP |
| `OPTIONS` | `/mcp` | Preflight CORS permissivo |

## Capacidades

O servidor anuncia somente ferramentas. Nao implementa prompts, resources,
sampling, roots ou logging MCP.

Ferramentas registradas:

- `wiki_search`
- `wiki_get_page`
- `wiki_get_section`
- `wiki_get_links`
- `wiki_get_metadata`

## Ciclo de requisicao

1. Axum aplica CORS e rate limit.
2. `StreamableHttpService` processa o JSON-RPC sem sessao persistente.
3. O macro `#[tool_router]` valida o shape do schema.
4. O handler aplica validacao semantica e chama `WikiService`.
5. `Json<T>` retorna conteudo estruturado e textual.

Antes da publicacao, os schemas gerados sao normalizados para remover formatos
numericos especificos de Rust como `uint32` e `uint64`. Os campos continuam
como `type: integer` e preservam restricoes como `minimum`, evitando warnings
em validadores JSON Schema que nao conhecem esses formatos.

## Postura publica

- Bind padrao `0.0.0.0`.
- Sem autenticacao.
- Qualquer origem CORS.
- `Host` e `Origin` nao sao validados pelo transporte.
- Sem limite de bytes no body MCP.
- Rate limit padrao de 60 requisicoes por minuto por IP.

Essa configuracao privilegia conexao simples para uma comunidade publica. O
operador deve usar TLS, monitoramento e protecoes externas quando necessario.

## Fora do escopo

- Transporte `stdio`.
- Estado de sessao persistente.
- Escrita ou operacoes autenticadas na wiki.
- Ferramentas de mods.
