# Use Quando Desenvolver MCP

## Limites de responsabilidade

Handlers MCP devem:

- Validar e limitar o input.
- Chamar um servico de dominio.
- Converter o resultado para o contrato MCP.
- Converter falhas para erros publicos seguros.

Handlers nao devem montar requisicoes MediaWiki diretamente nem conter regras de
cache, retry ou resolucao de redirects.

## Ferramentas planejadas da wiki

| Ferramenta | Finalidade |
| --- | --- |
| `wiki_search` | Pesquisar paginas com paginacao limitada |
| `wiki_get_page` | Obter pagina em texto, HTML processado ou wikitext |
| `wiki_get_section` | Obter uma secao, inclusive apos redirect |
| `wiki_get_links` | Listar links relevantes de uma pagina |
| `wiki_get_metadata` | Obter revisao, categorias, URL e licenca |

## Ferramentas candidatas de mods

`mods_search`, `mods_get`, `mods_list_files` e `mods_get_requirements` sao apenas
nomes candidatos. Nao registre nem estabilize seus schemas antes da escolha do
provedor.

## Contratos

- Defina schemas de entrada estritos e documente defaults e limites.
- Paginacao deve possuir teto do lado do servidor.
- Formatos aceitos devem ser enums, nao strings livres.
- Respostas devem distinguir conteudo da fonte de mensagens do servidor.
- Conteudo externo deve carregar `source`, `canonical_url` e atribuicao.
- Quando disponivel, inclua ID e timestamp da revisao.
- Informe titulo solicitado e titulo canonico quando houver redirect.

## Erros

Separe pelo menos estas categorias:

- Input invalido.
- Recurso nao encontrado.
- Limite excedido.
- Timeout da fonte.
- Fonte temporariamente indisponivel.
- Resposta inesperada da fonte.
- Erro interno.

Nao retorne stack traces, URLs internas de mock, headers ou corpos brutos.

## Transporte HTTP planejado

- Endpoint MCP: `/mcp`.
- Health check: `/health`.
- Bind e porta: configurados por ambiente.
- Autenticacao: **TBD**; ate ser definida, nao presuma exposicao segura a
  Internet.

## Testes

- Registro e listagem de ferramentas.
- Validacao de schema e limites.
- Conversao de cada classe de erro.
- Presenca de atribuicao nas respostas.
- Ausencia de detalhes internos nos erros publicos.
