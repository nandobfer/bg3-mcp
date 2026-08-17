# Modulo: MCP

## Objetivo

Expor ferramentas de consulta de Baldur's Gate 3 para clientes compativeis com
Model Context Protocol.

## Estado

O servidor e suas ferramentas estao **planejados**. SDK, versao e transporte
final ainda precisam de confirmacao tecnica.

## Fase inicial

A primeira fase e somente leitura e cobre a bg3.wiki:

- `wiki_search`
- `wiki_get_page`
- `wiki_get_section`
- `wiki_get_links`
- `wiki_get_metadata`

O dominio de mods entra somente depois da escolha e validacao do provedor.

## Transporte planejado

- Streamable HTTP.
- Endpoint MCP em `/mcp`.
- Health check local em `/health`.
- Host e porta configurados por ambiente.
- Autenticacao: **TBD**.

O endpoint nao deve ser exposto publicamente antes de uma decisao explicita de
autenticacao, proxy e limites operacionais.

## Respostas

As respostas devem ser estruturadas para que o cliente MCP formule a resposta
em linguagem natural. Conteudo da fonte e mensagens do servidor devem permanecer
distinguiveis.

Campos de proveniencia esperados:

- Nome da fonte.
- URL canonica.
- Titulo solicitado e canonico quando aplicavel.
- Revisao e timestamp quando disponiveis.
- Licenca ou URL de licenca.

## Fora do escopo inicial

- Escrita na wiki.
- Operacoes autenticadas na wiki.
- Crawling ou espelhamento.
- Instalacao e download de mods.
- Gerenciamento de load order.
