# Documentacao de Contexto para IA

Esta pasta concentra requisitos tecnicos, descobertas verificadas, contratos
planejados e regras de desenvolvimento do BG3 MCP. O `README.md` da raiz e
voltado ao usuario final.

## Estado da documentacao

Os documentos usam quatro estados:

- **Implementado**: comportamento presente no codigo e coberto por teste.
- **Verificado**: comportamento observado diretamente na fonte.
- **Planejado**: decisao de produto ou engenharia ainda nao implementada.
- **TBD**: depende de pesquisa, escolha tecnica ou informacao externa.

Documentacao planejada nao prova que o codigo correspondente existe.

## Ordem de leitura

1. Leia `use-sempre-que-desenvolver.md` em toda tarefa de implementacao.
2. Leia `use-diretrizes-do-projeto.md` para arquitetura ou novas dependencias.
3. Leia o guia especifico da tarefa.
4. Leia os modulos afetados em `modules/`.

## Guias

| Arquivo | Quando usar |
| --- | --- |
| `use-sempre-que-desenvolver.md` | Toda alteracao de codigo |
| `use-diretrizes-do-projeto.md` | Arquitetura, dependencias e decisoes tecnicas |
| `use-quando-desenvolver-mcp.md` | Ferramentas, transporte e erros MCP |
| `use-quando-integrar-fontes-http.md` | Clientes HTTP, resiliencia e novas fontes |
| `use-para-atualizar-containers.md` | Dockerfile, Compose e ambiente |

## Modulos

| Arquivo | Conteudo |
| --- | --- |
| `modules/wiki.md` | Contrato e descobertas da bg3.wiki |
| `modules/mods.md` | Contrato e integracao de leitura com o mod.io |
| `modules/mcp.md` | Superficie de ferramentas e transporte |
| `modules/infrastructure.md` | HTTP, cache, concorrencia, erros e tracing |

## Atualizacao

Atualize esta documentacao quando houver:

- Mudanca em contrato de ferramenta ou resposta.
- Nova variavel de ambiente.
- Nova fonte externa ou alteracao de politica da fonte.
- Decisao de SDK, transporte, cache, autenticacao ou deploy.
- Novo modulo de dominio.

Detalhes locais de implementacao sem efeito em contrato ou arquitetura nao
precisam ser documentados aqui.
