# Modulo: Wiki

## Objetivo

Consultar a bg3.wiki sob demanda para pesquisa, leitura de paginas e secoes,
links e metadados, sempre com atribuicao.

## Estado

As capacidades HTTP abaixo foram **verificadas em 17 de agosto de 2026**. O
cliente Rust e as ferramentas MCP ainda estao **planejados**.

## Endpoints verificados

| Interface | Endpoint | Uso |
| --- | --- | --- |
| Action API | `https://bg3.wiki/w/api.php` | Pesquisa, extracts, parse, links e revisoes |
| REST API | `https://bg3.wiki/w/rest.php/v1` | Paginas e wikitext bruto |
| Ajuda | `https://bg3.wiki/w/api.php?action=help` | Modulos habilitados |

A instalacao informou MediaWiki `1.43.9` e extensao Cargo `3.9.2`.

## Resultados verificados

- `GET` na Action API com `format=json` retorna JSON.
- `POST` funciona com `application/x-www-form-urlencoded`.
- `POST` com `application/json` nao interpreta a consulta e retorna a ajuda HTML.
- `list=search`, `prop=extracts`, `action=parse` e `prop=revisions` funcionam.
- `GET /page/{title}` retorna metadados e wikitext em `source`.
- `action=cargoquery` arbitrario retorna `permissiondenied`.

O projeto nao deve depender de consultas Cargo arbitrarias.

## Requisicoes de referencia

Pesquisa textual:

```bash
curl -G 'https://bg3.wiki/w/api.php' \
  -H 'User-Agent: BG3MCP/0.1 (contact: CHANGE_ME)' \
  --data-urlencode 'action=query' \
  --data-urlencode 'list=search' \
  --data-urlencode 'srsearch=Great Weapon Master' \
  --data-urlencode 'srlimit=5' \
  --data-urlencode 'format=json' \
  --data-urlencode 'formatversion=2'
```

Pagina como texto:

```bash
curl -G 'https://bg3.wiki/w/api.php' \
  -H 'User-Agent: BG3MCP/0.1 (contact: CHANGE_ME)' \
  --data-urlencode 'action=query' \
  --data-urlencode 'prop=extracts' \
  --data-urlencode 'explaintext=1' \
  --data-urlencode 'redirects=1' \
  --data-urlencode "titles=Baldur's Gate 3" \
  --data-urlencode 'format=json' \
  --data-urlencode 'formatversion=2'
```

Wikitext pela REST API:

```bash
curl 'https://bg3.wiki/w/rest.php/v1/page/Baldur%27s_Gate_3' \
  -H 'Accept: application/json' \
  -H 'User-Agent: BG3MCP/0.1 (contact: CHANGE_ME)'
```

## Pesquisa e sanitizacao

Snippets de pesquisa podem conter tags como `<span class="searchmatch">`.
Remova ou converta essa marcacao antes de retornar o resultado MCP.

## Redirects e fragmentos

`Great Weapon Master` redireciona para `Feats#Great Weapon Master`. Com
`redirects=1`, a API informa `to` e `tofragment`.

O servico deve:

1. Detectar `tofragment`.
2. Consultar as secoes da pagina canonica.
3. Encontrar a secao correspondente.
4. Fazer parse apenas da secao quando possivel.
5. Retornar titulo solicitado, titulo canonico e fragmento.

Uma extracao comum da pagina canonica nao substitui essa resolucao, pois pode
retornar apenas a introducao de `Feats`.

## Uso responsavel

`robots.txt` declara `Disallow` para `/w/api.php` e `/w/rest.php`. Os endpoints
responderam aos testes, mas nao devem ser usados para crawling indiscriminado.
O uso inicial e somente sob demanda. Antes de uso publico ou em volume, confirme
a politica com os mantenedores.

Controles obrigatorios:

- `User-Agent` com versao e contato.
- Cache para consultas repetidas.
- Timeout e concorrencia limitada.
- Backoff para `429` e falhas transitorias.
- `maxlag` quando aplicavel.
- Limite de tamanho de resposta.
- Logs sem credenciais ou conteudo sensivel.

## Licenca e atribuicao

A API informa `CC BY-NC-SA 4.0 or CC BY-SA 4.0` e referencia
`https://bg3.wiki/wiki/bg3wiki:Copyrights`. A licenca exata pode depender do
conteudo.

Toda resposta de leitura deve incluir, quando disponivel:

- Titulo solicitado e canonico.
- URL canonica.
- ID e timestamp da revisao.
- Fonte `bg3.wiki`.
- Informacao ou URL de licenca.

A conformidade deve ser revisada antes de armazenamento, redistribuicao ou
deploy publico.
