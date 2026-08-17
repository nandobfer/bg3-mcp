# Diretrizes do Projeto

## Objetivo

Construir um servidor MCP em Rust para consulta de informacoes de Baldur's Gate
3. A primeira fonte e a bg3.wiki. Um provedor de mods sera integrado somente
depois de sua API, licenca e regras de uso serem definidas.

## Arquitetura planejada

```text
MCP client
    |
    v
MCP transport
    |
    +-- Wiki tool handlers --> Wiki service --> bg3.wiki client
    |
    +-- Mods tool handlers --> Mods service --> provider TBD

Shared infrastructure:
HTTP client | cache | concurrency | errors | tracing | attribution
```

O servidor e unico, mas os dominios de wiki e mods permanecem separados. Uma
abstracao compartilhada so deve existir quando o codigo demonstrar reutilizacao
real.

## Dependencias candidatas

Estas dependencias ainda devem ser confirmadas junto da escolha do SDK MCP:

- `tokio`: runtime assincrono.
- `reqwest`: cliente HTTP.
- `serde` e `serde_json`: contratos e serializacao.
- `tracing`: logs e telemetria.
- `thiserror`: erros tipados.
- SDK MCP Rust: **TBD**.
- Cache em memoria ou persistente: **TBD**.

Versoes devem ser fixadas no `Cargo.lock`. Dependencias novas precisam ter uso
concreto e manutencao ativa.

## Transporte

O deploy em container assume Streamable HTTP como direcao inicial e reserva
`/mcp` para o endpoint MCP e `/health` para saude. A escolha final depende do SDK
e dos clientes que serao suportados.

Suporte a `stdio` so deve ser adicionado se houver necessidade concreta. Nao
implemente dois transportes preventivamente.

## Configuracao

O processo deve ser configurado por ambiente:

| Variavel | Finalidade |
| --- | --- |
| `BG3_WIKI_BASE_URL` | URL-base da bg3.wiki ou servidor mockado |
| `BG3_MCP_USER_AGENT` | Identificacao enviada a fontes externas |
| `BG3_MCP_HTTP_TIMEOUT_SECS` | Timeout de requisicoes externas |
| `BG3_MCP_MAX_CONCURRENCY` | Concorrencia maxima por fonte |
| `BG3_MCP_CACHE_TTL_SECS` | TTL padrao do cache |
| `BG3_MCP_LOG` | Filtro de logs |
| `BG3_MCP_HOST` | Interface do servidor HTTP |
| `BG3_MCP_PORT` | Porta do servidor HTTP |
| `BG3_MCP_TRANSPORT` | Transporte selecionado |

Credenciais futuras do provedor de mods devem usar ambiente ou secrets do
orquestrador.

## Etapas de implementacao

1. Pesquisar e escolher o SDK MCP e confirmar o transporte.
2. Criar o crate Rust e carregar configuracao tipada.
3. Definir schemas das ferramentas da wiki.
4. Implementar o cliente MediaWiki e seus modelos.
5. Implementar pesquisa, pagina, secao, links e metadados.
6. Adicionar cache, timeout, concorrencia e erros normalizados.
7. Registrar as ferramentas no servidor MCP.
8. Adicionar testes com servidor mockado.
9. Validar imagem e Compose com smoke test.
10. Avaliar o provedor de mods antes de implementar esse dominio.

## Criterios de aceite iniciais

- Pesquisa e leitura da bg3.wiki funcionam por ferramentas MCP.
- Redirects e fragmentos resolvem a secao correta.
- Respostas incluem URL, fonte e atribuicao.
- Falhas externas viram erros MCP claros e seguros.
- Requisicoes possuem timeout, concorrencia limitada e cache.
- Nao ha crawling ou espelhamento da wiki.
- Testes automatizados usam mocks.
- O servidor inicia e passa no health check via Compose.

## Decisoes pendentes

- SDK MCP Rust e versao.
- Confirmacao do Streamable HTTP e clientes suportados.
- Autenticacao e exposicao publica.
- Implementacao e persistencia do cache.
- Contato oficial do `User-Agent`.
- Limites operacionais acordados com a bg3.wiki.
- Provedor e escopo de mods.
- Registry da imagem e ambiente final de hospedagem.
