# BG3 MCP

Servidor MCP para consultar dados de Baldur's Gate 3 a partir de duas fontes:

1. [bg3.wiki](https://bg3.wiki/), para conteúdo enciclopédico sobre o jogo.
2. Um catálogo de mods ainda a definir, para pesquisa e navegação de mods.

Este documento registra as descobertas técnicas já verificadas e orienta o
planejamento inicial da implementação em Rust e do deploy com Docker Compose.
As decisões que dependem da futura fonte de mods ou de uma avaliação técnica
adicional estão marcadas como `TBD`.

## Objetivos

- Expor ferramentas MCP para pesquisar e consultar conteúdo do BG3.
- Retornar dados estruturados com atribuição e URL da fonte.
- Manter os domínios de wiki e mods separados dentro do mesmo servidor.
- Executar o servidor de forma reproduzível por meio de Docker Compose.
- Respeitar licenças, limites operacionais e políticas das fontes consultadas.

O MCP deverá recuperar e normalizar informações. A formulação da resposta em
linguagem natural normalmente caberá ao cliente MCP e ao modelo que consumir as
ferramentas.

## Escopo inicial

### Incluído

- Pesquisa textual na bg3.wiki.
- Leitura de páginas como texto, HTML processado ou wikitext.
- Consulta de metadados, links, revisões e seções.
- Resolução de páginas que redirecionam para outras páginas ou fragmentos.
- Cache, timeouts, limitação de concorrência e tratamento de erros.
- Pesquisa e navegação de mods após a definição da segunda fonte.
- Execução local e deploy com Docker Compose.

### Fora do escopo inicial

- Editar conteúdo da wiki.
- Operações autenticadas na wiki.
- Fazer crawling ou espelhamento integral da bg3.wiki.
- Instalar mods automaticamente.
- Gerenciar load order ou modificar uma instalação local do jogo.
- Baixar arquivos de mods, até que isso seja solicitado e permitido pela fonte.

## Descobertas sobre a bg3.wiki

Os testes abaixo foram executados em 17 de agosto de 2026. A bg3.wiki utiliza
MediaWiki `1.43.9` e expõe as APIs padrão da plataforma.

### Endpoints

| Interface | Endpoint | Uso principal |
| --- | --- | --- |
| MediaWiki Action API | `https://bg3.wiki/w/api.php` | Pesquisa, extração, parse, links, revisões e metadados |
| MediaWiki REST API | `https://bg3.wiki/w/rest.php/v1` | Leitura simplificada de páginas e wikitext bruto |
| Ajuda da Action API | `https://bg3.wiki/w/api.php?action=help` | Documentação dos módulos habilitados |

### Resultados verificados

| Teste | Resultado |
| --- | --- |
| `GET` na Action API com `format=json` | `HTTP 200` e `application/json; charset=utf-8` |
| `POST` com `application/x-www-form-urlencoded` | `HTTP 200` e resposta JSON válida |
| `POST` com corpo `application/json` | O corpo não foi interpretado; a API devolveu a página HTML de ajuda |
| Pesquisa com `list=search` | Funcionou e retornou resultados, IDs, títulos e snippets |
| Extração com `prop=extracts` | Funcionou com texto simples |
| Parse com `action=parse` | Funcionou e retornou HTML e metadados da página |
| Revisões com `prop=revisions` | Funcionou e retornou wikitext, ID e data da revisão |
| REST `GET /page/{title}` | Funcionou e retornou metadados e wikitext em `source` |
| Consulta arbitrária pela extensão Cargo | Negada com `permissiondenied` |

A consulta de informações do site retornou, entre outros campos:

```json
{
  "sitename": "bg3.wiki",
  "generator": "MediaWiki 1.43.9",
  "server": "https://bg3.wiki"
}
```

Uma consulta REST à página `Baldur's Gate 3` retornou uma estrutura equivalente
a:

```json
{
  "id": 3500,
  "key": "Baldur's_Gate_3",
  "title": "Baldur's Gate 3",
  "content_model": "wikitext",
  "latest": {
    "id": 397217,
    "timestamp": "2026-06-27T00:30:50Z"
  },
  "source": "..."
}
```

### Exemplos reproduzíveis

As requisições de produção deverão usar um `User-Agent` que identifique o
projeto e forneça uma forma de contato.

#### Informações do site

```bash
curl -G 'https://bg3.wiki/w/api.php' \
  -H 'User-Agent: BG3MCP/0.1 (contact: TBD)' \
  --data-urlencode 'action=query' \
  --data-urlencode 'meta=siteinfo' \
  --data-urlencode 'siprop=general' \
  --data-urlencode 'format=json' \
  --data-urlencode 'formatversion=2'
```

#### Pesquisa textual

```bash
curl -G 'https://bg3.wiki/w/api.php' \
  -H 'User-Agent: BG3MCP/0.1 (contact: TBD)' \
  --data-urlencode 'action=query' \
  --data-urlencode 'list=search' \
  --data-urlencode 'srsearch=Great Weapon Master' \
  --data-urlencode 'srlimit=5' \
  --data-urlencode 'format=json' \
  --data-urlencode 'formatversion=2'
```

Os snippets de pesquisa podem conter tags HTML como `<span
class="searchmatch">`. O servidor deverá removê-las ou convertê-las para uma
representação segura antes de retornar o resultado ao cliente MCP.

#### Página como texto simples

```bash
curl -G 'https://bg3.wiki/w/api.php' \
  -H 'User-Agent: BG3MCP/0.1 (contact: TBD)' \
  --data-urlencode 'action=query' \
  --data-urlencode 'prop=extracts' \
  --data-urlencode 'explaintext=1' \
  --data-urlencode 'redirects=1' \
  --data-urlencode "titles=Baldur's Gate 3" \
  --data-urlencode 'format=json' \
  --data-urlencode 'formatversion=2'
```

#### Página via POST

```bash
curl -X POST 'https://bg3.wiki/w/api.php' \
  -H 'User-Agent: BG3MCP/0.1 (contact: TBD)' \
  -H 'Content-Type: application/x-www-form-urlencoded' \
  --data-urlencode 'action=query' \
  --data-urlencode 'prop=extracts' \
  --data-urlencode 'explaintext=1' \
  --data-urlencode 'redirects=1' \
  --data-urlencode "titles=Baldur's Gate 3" \
  --data-urlencode 'format=json' \
  --data-urlencode 'formatversion=2'
```

Embora a resposta seja JSON, os parâmetros de um `POST` devem ser enviados
como formulário. Um corpo com `Content-Type: application/json` não foi aceito
como consulta pela instalação testada.

#### Wikitext pela REST API

```bash
curl 'https://bg3.wiki/w/rest.php/v1/page/Baldur%27s_Gate_3' \
  -H 'Accept: application/json' \
  -H 'User-Agent: BG3MCP/0.1 (contact: TBD)'
```

O wikitext está no campo `source`. Esse formato preserva templates e marcações
da wiki, mas requer processamento adicional para produzir texto legível.

#### HTML processado e estrutura da página

```bash
curl -G 'https://bg3.wiki/w/api.php' \
  -H 'User-Agent: BG3MCP/0.1 (contact: TBD)' \
  --data-urlencode 'action=parse' \
  --data-urlencode 'page=Great Weapon Master' \
  --data-urlencode 'prop=text|sections|categories|links' \
  --data-urlencode 'format=json' \
  --data-urlencode 'formatversion=2'
```

### Redirecionamentos e fragmentos

Nem todo título corresponde a uma página independente. O título `Great Weapon
Master`, por exemplo, redireciona para `Feats#Great Weapon Master`.

Com `redirects=1`, a Action API informa a página de destino e o fragmento:

```json
{
  "from": "Great Weapon Master",
  "to": "Feats",
  "tofragment": "Great Weapon Master"
}
```

Uma extração comum pode devolver apenas a introdução da página `Feats`, e não a
seção desejada. O cliente da wiki deverá:

1. Detectar `tofragment` na resposta de redirecionamento.
2. Consultar as seções da página de destino.
3. Encontrar a seção correspondente ao fragmento.
4. Fazer o parse apenas da seção encontrada, quando possível.
5. Informar o título solicitado e o destino canônico na resposta MCP.

### Cargo

A extensão Cargo `3.9.2` está instalada, mas uma tentativa de executar
`action=cargoquery` retornou:

```json
{
  "error": {
    "code": "permissiondenied",
    "info": "You don't have permission to run arbitrary Cargo queries."
  }
}
```

O projeto não deve depender de consultas Cargo arbitrárias. Se uma futura
necessidade exigir dados estruturados do Cargo, será preciso verificar quais
consultas são permitidas ou conversar com os mantenedores da wiki.

### Robots, carga e uso responsável

O arquivo `https://bg3.wiki/robots.txt` contém:

```text
User-agent: *
Disallow: /w/api.php
Disallow: /w/rest.php
```

Essas diretivas se destinam a crawlers e não representam um bloqueio técnico:
as APIs responderam normalmente aos testes. Ainda assim, elas indicam que não
devemos tratar os endpoints como uma fonte para crawling indiscriminado.

O uso inicial deverá ser limitado a consultas sob demanda. Antes de disponibilizar
um serviço público ou executar coleta em volume, o projeto deverá consultar os
mantenedores da bg3.wiki e confirmar o uso aceitável.

Controles operacionais previstos:

- `User-Agent` descritivo com versão e contato.
- Cache de resultados para evitar requisições repetidas.
- Timeout de conexão e de resposta.
- Limite de concorrência por fonte.
- Backoff exponencial para falhas transitórias e respostas `429` ou `5xx`.
- Suporte ao parâmetro `maxlag` da Action API quando aplicável.
- Limite de tamanho para respostas e páginas processadas.
- Logs sem conteúdo sensível ou credenciais.

### Licença e atribuição

A API informa `CC BY-NC-SA 4.0 or CC BY-SA 4.0` e aponta para a página de
[copyright da bg3.wiki](https://bg3.wiki/wiki/bg3wiki:Copyrights). O significado
exato da licença aplicável pode depender do conteúdo consultado.

Toda ferramenta de leitura deverá retornar metadados de atribuição, incluindo:

- Título da página.
- URL canônica.
- ID e timestamp da revisão, quando disponíveis.
- Nome da fonte (`bg3.wiki`).
- Informação ou URL de licença.

A conformidade final deverá ser revisada antes de oferecer o serviço
publicamente, especialmente se respostas forem armazenadas ou redistribuídas.

## Superfície MCP preliminar

Os nomes e schemas abaixo são propostas e serão confirmados durante o plano de
implementação.

### Wiki

| Ferramenta | Finalidade |
| --- | --- |
| `wiki_search` | Pesquisar páginas por texto e retornar resultados paginados |
| `wiki_get_page` | Obter uma página em texto, HTML ou wikitext |
| `wiki_get_section` | Obter uma seção específica, inclusive após redirecionamento |
| `wiki_get_links` | Listar links relevantes presentes em uma página |
| `wiki_get_metadata` | Obter revisão, categorias, URL, licença e outros metadados |

Todas as respostas deverão distinguir conteúdo da fonte de mensagens geradas
pelo servidor e incluir atribuição.

### Mods

Ferramentas candidatas, pendentes de validação com a fonte:

| Ferramenta | Finalidade preliminar |
| --- | --- |
| `mods_search` | Pesquisar mods com paginação e filtros |
| `mods_get` | Consultar detalhes, descrição, autor e versão de um mod |
| `mods_list_files` | Listar arquivos e versões disponíveis |
| `mods_get_requirements` | Consultar dependências e requisitos conhecidos |

Não serão definidos schemas definitivos antes de conhecermos a API e as regras
do provedor de mods.

## Arquitetura preliminar

O projeto terá um único servidor MCP, mas serviços de domínio separados para
wiki e mods. Não será criada uma abstração genérica entre as duas fontes sem um
caso concreto de reutilização.

```text
MCP client
    |
    v
MCP transport
    |
    +-- Wiki tool handlers ----> Wiki service ----> bg3.wiki client
    |
    +-- Mods tool handlers ----> Mods service ----> mods provider client (TBD)
                                  
Shared infrastructure:
HTTP client | cache | rate limits | errors | tracing | attribution
```

### Módulos Rust previstos

Uma estrutura inicial possível:

```text
src/
  main.rs
  config.rs
  error.rs
  mcp/
    mod.rs
    wiki_tools.rs
    mods_tools.rs
  wiki/
    mod.rs
    client.rs
    models.rs
    service.rs
  mods/
    mod.rs
    client.rs
    models.rs
    service.rs
  infrastructure/
    http.rs
    cache.rs
    rate_limit.rs
```

Essa divisão é preliminar. A implementação deve começar menor e criar módulos
somente quando houver código suficiente para justificar cada separação.

### Dependências candidatas

- `tokio` para execução assíncrona.
- `reqwest` para HTTP.
- `serde` e `serde_json` para serialização.
- `tracing` para logs e telemetria.
- `thiserror` para erros de domínio.
- SDK MCP para Rust: `TBD` após pesquisa de maturidade e compatibilidade.
- Cache persistente ou em memória: `TBD` após definir o perfil de uso.

As versões só serão fixadas após a escolha do SDK MCP e do transporte.

### Transporte MCP

O transporte ainda será definido. Como haverá deploy em container, um
transporte remoto por HTTP tende a ser mais apropriado que `stdio`, mas a escolha
dependerá do SDK, do cliente MCP e do ambiente onde o Compose será executado.

Opções a avaliar:

- Streamable HTTP para acesso remoto ou pela rede do Compose.
- `stdio` para desenvolvimento local e integração com clientes que iniciam o
  processo diretamente.
- Suporte aos dois modos somente se houver uma necessidade concreta.

Autenticação e exposição pública também estão `TBD`. Até isso ser definido, o
serviço deverá escutar apenas em uma interface ou rede considerada segura.

## Docker e Docker Compose

### Imagem

- Build multi-stage para compilar o binário Rust.
- Imagem final mínima e sem toolchain de compilação.
- Execução como usuário sem privilégios.
- Binário e certificados CA como únicos requisitos de runtime, quando possível.
- Health check compatível com o transporte escolhido.
- Versões de imagens fixadas explicitamente.

### Compose inicial

O primeiro `compose.yaml` deverá possuir somente o serviço MCP:

```text
services:
  bg3-mcp:
    build: .
    environment: ...
    ports: ...
    healthcheck: ...
    restart: unless-stopped
```

Não serão adicionados Redis, banco de dados ou outros containers até existir uma
necessidade concreta. Se o cache precisar sobreviver a reinícios, será avaliado
um cache local persistente com volume antes de introduzir outro serviço.

### Configuração prevista

Nomes definitivos serão estabelecidos durante a implementação. Configurações
prováveis:

| Configuração | Finalidade |
| --- | --- |
| `BG3_WIKI_BASE_URL` | Endpoint base da wiki, permitindo testes com mock server |
| `BG3_MCP_USER_AGENT` | Identificação enviada às fontes |
| `BG3_MCP_HTTP_TIMEOUT` | Timeout das requisições externas |
| `BG3_MCP_MAX_CONCURRENCY` | Limite de consultas simultâneas por fonte |
| `BG3_MCP_CACHE_TTL` | Duração padrão do cache |
| `BG3_MCP_LOG` | Nível ou filtro de logs |
| `BG3_MCP_BIND` | Interface e porta do transporte HTTP, se usado |
| Credenciais de mods | `TBD`, sempre via secret ou ambiente |

Nenhuma credencial deverá ser armazenada no repositório ou incorporada à
imagem.

## Fonte de mods pendente

Antes de planejar a integração de mods em detalhe, são necessárias estas
informações:

- Site, API ou ferramenta que fornecerá os dados.
- Documentação e estabilidade da API.
- Método de autenticação e armazenamento das credenciais.
- Rate limits e regras de uso automatizado.
- Licença e condições de redistribuição.
- Modelo de paginação, pesquisa e filtros.
- Identificadores estáveis de jogos, mods, arquivos e versões.
- Representação de compatibilidade, dependências e requisitos.
- Disponibilidade de changelogs, imagens e métricas.
- Necessidade de consulta apenas ou também download de arquivos.

Após receber essas informações, esta seção deverá registrar testes equivalentes
aos feitos para a bg3.wiki antes da criação do cliente Rust.

## Etapas de implementação

1. Pesquisar e escolher o SDK MCP Rust e o transporte.
2. Definir schemas de entrada e saída das ferramentas da wiki.
3. Criar o projeto Rust e sua configuração.
4. Implementar o cliente MediaWiki com respostas tipadas.
5. Implementar pesquisa, leitura, metadados e tratamento de redirects.
6. Adicionar cache, limites, timeout e erros normalizados.
7. Expor as ferramentas pelo servidor MCP.
8. Adicionar testes unitários e testes HTTP com servidor mockado.
9. Criar Dockerfile multi-stage e `compose.yaml`.
10. Executar testes de integração e um smoke test pelo Compose.
11. Avaliar e documentar a fonte de mods.
12. Implementar as ferramentas de mods sem acoplá-las ao domínio da wiki.
13. Revisar licenças, atribuição, segurança e política de uso antes do deploy
    público.

## Critérios de aceite iniciais

- O MCP pesquisa a bg3.wiki e lê páginas e seções.
- Redirecionamentos e fragmentos são tratados corretamente.
- As respostas possuem conteúdo estruturado, URL e atribuição.
- Falhas externas são convertidas em erros MCP claros e seguros.
- Requisições possuem timeout, limite de concorrência e cache.
- O servidor não realiza crawling ou sincronização integral da wiki.
- Testes não dependem da disponibilidade da bg3.wiki, salvo testes manuais ou
  explicitamente marcados como integração externa.
- O servidor inicia e passa pelo health check usando Docker Compose.
- A arquitetura aceita o domínio de mods sem forçar um modelo comum prematuro.

## Decisões pendentes

- SDK MCP Rust e sua versão.
- Transporte MCP para desenvolvimento e deploy.
- Estratégia de autenticação do servidor.
- Política e implementação do cache.
- Endereço de contato usado no `User-Agent`.
- Limites operacionais acordados com a bg3.wiki.
- Fonte, API e escopo funcional de mods.
- Ambiente final de hospedagem e forma de exposição do container.
