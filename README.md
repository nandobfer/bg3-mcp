# BG3 MCP

Servidor [Model Context Protocol (MCP)](https://modelcontextprotocol.io/) para
consultar informacoes de Baldur's Gate 3 a partir de fontes externas.

O objetivo e permitir que assistentes e outros clientes MCP pesquisem conteudo
do jogo, consultem paginas e encontrem informacoes relevantes sem depender de
uma interface web propria.

## Estado do projeto

O BG3 MCP esta em desenvolvimento inicial e ainda nao possui uma versao
executavel. A primeira integracao sera com a [bg3.wiki](https://bg3.wiki/). Uma
fonte de mods sera adicionada depois que o provedor e suas regras de uso forem
definidos.

## O que o servidor oferecera

- Pesquisa de paginas da bg3.wiki.
- Leitura de paginas e secoes especificas.
- Consulta de links, revisoes e metadados.
- Tratamento de paginas redirecionadas.
- Respostas com URL e atribuicao da fonte.
- Pesquisa de mods em uma etapa futura.

O servidor recuperara e organizara os dados. A resposta em linguagem natural
sera produzida normalmente pelo assistente conectado ao MCP.

## O que nao faz parte do escopo inicial

- Editar a bg3.wiki.
- Espelhar ou coletar a wiki em massa.
- Instalar ou baixar mods.
- Alterar load order.
- Modificar uma instalacao local de Baldur's Gate 3.

## Executar com Docker Compose

O container estara disponivel quando a primeira versao do servidor for
implementada. A configuracao preparada no repositorio usara este fluxo:

1. Edite `.env` e substitua os valores marcados com `CHANGE_ME`.
2. Escolha a porta em `BG3_MCP_PORT`.
3. Inicie o servico:

```bash
docker compose up --build -d
```

4. Verifique o estado:

```bash
docker compose ps
```

O endpoint MCP planejado sera:

```text
http://localhost:<BG3_MCP_PORT>/mcp
```

O health check planejado sera:

```text
http://localhost:<BG3_MCP_PORT>/health
```

Esses endpoints ainda nao estao implementados.

## Como conectar

Depois que o servidor estiver disponivel, use a URL `/mcp` em um cliente com
suporte a MCP remoto. Um formato comum de configuracao e:

```json
{
  "mcpServers": {
    "bg3": {
      "url": "http://localhost:3000/mcp"
    }
  }
}
```

O formato exato varia entre clientes. Troque `3000` pela porta configurada em
`BG3_MCP_PORT`.

## Fontes e atribuicao

O conteudo da wiki pertence aos respectivos autores e esta sujeito as licencas
e regras publicadas pela [bg3.wiki](https://bg3.wiki/). As respostas do servidor
deverao preservar a fonte e a URL canonica do conteudo consultado.

O projeto realiza consultas sob demanda e nao tem como objetivo rastrear ou
espelhar integralmente a wiki.

## Desenvolvimento

Requisitos tecnicos, arquitetura, investigacoes e contratos planejados ficam em
[`aicontext/`](aicontext/README.md). Colaboradores e agentes de IA devem ler
[`AGENTS.md`](AGENTS.md) antes de alterar o projeto.
