# Modulo: Wiki

## Estado

As APIs foram **verificadas em 17 de agosto de 2026**. O cliente Rust e as cinco
ferramentas MCP estao **implementados** e cobertos por testes com mock.

## Endpoints

| Interface | Endpoint | Uso implementado |
| --- | --- | --- |
| Action API | `/w/api.php` | Pesquisa, extracts, parse, links, revisoes e categorias |
| REST API | `/w/rest.php/v1/page/{title}` | Wikitext bruto |

A URL-base e configuravel para testes. A instalacao verificada informou
MediaWiki `1.43.9` e Cargo `3.9.2`.

## Action API

Todas as consultas usam `POST application/x-www-form-urlencoded`,
`format=json`, `formatversion=2` e `maxlag=5`. JSON no body nao e aceito pela
instalacao observada.

Mapeamento:

| Operacao | Modulo MediaWiki |
| --- | --- |
| Pesquisa | `action=query&list=search` |
| Resolucao | `action=query&redirects=1` |
| Texto | `prop=extracts&explaintext=1` |
| HTML e secoes | `action=parse` |
| Links | `prop=links&plnamespace=0` |
| Metadados | `prop=info|revisions|categories` |

Consultas Cargo arbitrarias retornaram `permissiondenied` e nao sao usadas.

## Pesquisa

`wiki_search` aceita de 1 a 20 resultados e usa `sroffset` como cursor. Snippets
HTML sao convertidos para texto antes da resposta. Cada resultado recebe URL de
pagina construida na origem configurada.

## Paginas e formatos

- `text`: extract ou texto extraido de HTML de fragmento.
- `html`: parse sanitizado com `ammonia`.
- `wikitext`: campo `source` da REST API ou parse de secao MediaWiki.

Nao ha truncamento nem limite de tamanho de download ou conteudo retornado.

## Redirects e fragments

`redirects=1` fornece titulo canonico e `tofragment`. O servico preserva titulo
solicitado, destino e fragmento na resposta.

A verificacao real mostrou que `Great Weapon Master` redireciona para
`Feats#Great Weapon Master`, mas o destino e um anchor de linha de tabela e nao
uma entrada de `prop=sections`. A resolucao implementada segue esta ordem:

1. Procurar por indice, heading ou anchor em `prop=sections`.
2. Se encontrado, fazer parse apenas da secao.
3. Caso contrario, fazer parse do HTML completo e localizar o elemento por `id`.
4. Se o anchor estiver em linha com `rowspan`, retornar todo o grupo de linhas.
5. Para texto, converter o fragmento encontrado; para HTML, sanitiza-lo.
6. Wikitext de anchor arbitrario retorna erro claro, pois MediaWiki nao expoe
   uma secao de wikitext equivalente; headings MediaWiki continuam suportados.

## Links e metadados

Links usam `plcontinue`, namespace principal e limite de 1 a 100 itens.
Metadados incluem ID, timestamp da ultima revisao, content model, categorias,
URL canonica e indicador de categorias completas.

## Atribuicao

Toda resposta inclui:

- Fonte `bg3.wiki` e URL-base.
- URL de copyright.
- Licenca indicada como dependente do conteudo.
- Titulo solicitado e canonico quando aplicavel.
- Revisao e timestamp em respostas de pagina e metadados.

## Uso responsavel

`robots.txt` declara `Disallow` para os endpoints. O projeto faz apenas consultas
sob demanda, com cache, timeout, concorrencia conservadora, `maxlag` e retry.
Crawling, espelhamento e coleta em massa permanecem proibidos.

Antes de operar em volume publico, confirme a politica com os mantenedores da
bg3.wiki. O projeto decidiu nao limitar bytes; monitore consumo de memoria.
