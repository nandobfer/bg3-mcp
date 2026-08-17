# BG3 MCP

Servidor [Model Context Protocol (MCP)](https://modelcontextprotocol.io/) publico
e comunitario para consultar informacoes de Baldur's Gate 3 na
[bg3.wiki](https://bg3.wiki/).

O projeto e gratuito, open source e nao oficial. Ele nao e afiliado a Larian
Studios nem a equipe da bg3.wiki.

## O que ele oferece

- Pesquisa textual na bg3.wiki.
- Leitura de paginas em texto, HTML sanitizado ou wikitext.
- Leitura de secoes e fragmentos, inclusive redirects como `Great Weapon Master`.
- Listagem paginada de links internos.
- Consulta de revisoes, categorias, URL canonica e licenca.
- Respostas estruturadas com atribuicao da fonte.

O servidor retorna o conteudo no idioma original da wiki. A formulacao ou
traducao da resposta em linguagem natural cabe ao cliente MCP conectado.

## Ferramentas MCP

| Ferramenta | Finalidade |
| --- | --- |
| `wiki_search` | Pesquisar paginas por texto |
| `wiki_get_page` | Obter uma pagina em texto, HTML ou wikitext |
| `wiki_get_section` | Obter uma secao ou fragment especifico |
| `wiki_get_links` | Listar links internos de uma pagina |
| `wiki_get_metadata` | Obter revisao, categorias, URL e licenca |

## Executar com Docker Compose

1. Edite `.env` e substitua `CHANGE_ME` por um email ou URL de contato.
2. Escolha a porta em `BG3_MCP_PORT`.
3. Inicie o servidor:

```bash
docker compose up --build -d
```

4. Confira o estado:

```bash
docker compose ps
curl http://localhost:3000/health
```

Os endpoints padrao sao:

```text
MCP:    http://localhost:3000/mcp
Saude:  http://localhost:3000/health
```

Troque `3000` pela porta definida em `.env`.

## Como conectar

Use a URL `/mcp` em um cliente compativel com Streamable HTTP. Um formato comum
de configuracao e:

```json
{
  "mcpServers": {
    "bg3": {
      "url": "http://localhost:3000/mcp"
    }
  }
}
```

O formato exato varia entre clientes MCP.

## Acesso publico

O servidor escuta em `0.0.0.0`, nao exige autenticacao e permite CORS de qualquer
origem. A aplicacao nao valida os headers `Host` ou `Origin` e nao limita o
tamanho de bodies MCP, downloads ou conteudo retornado.

Essa configuracao facilita conexoes comunitarias, mas permite consumo elevado de
memoria, abuso e negacao de servico. Ao publicar na Internet:

- Use TLS por meio de um reverse proxy.
- Monitore memoria, latencia e volume de requisicoes.
- Ajuste `BG3_MCP_RATE_LIMIT_PER_MINUTE` para a capacidade do host.
- Confirme com a bg3.wiki o volume de uso aceitavel.
- Considere limites no proxy se o ambiente exigir protecao adicional.

O rate limit da aplicacao e por IP da conexao. Quando existe um reverse proxy,
todos os clientes podem ser vistos como o mesmo IP, pois o servidor nao confia
automaticamente em headers encaminhados.

## Uso responsavel

O servidor faz consultas sob demanda e nao rastreia nem espelha integralmente a
wiki. O conteudo pertence aos respectivos autores e esta sujeito as licencas e
regras da [bg3.wiki](https://bg3.wiki/). Cada resposta inclui a fonte e a URL de
licenca.

## Fora do escopo

- Editar a bg3.wiki.
- Operacoes autenticadas na wiki.
- Crawling ou espelhamento em massa.
- Traducao automatica.
- Instalar ou baixar mods.
- Alterar load order ou uma instalacao local do jogo.

## Desenvolvimento

Requisitos tecnicos, arquitetura e contratos ficam em
[`aicontext/`](aicontext/README.md). Colaboradores e agentes de IA devem ler
[`AGENTS.md`](AGENTS.md) antes de alterar o projeto.

Validacao completa:

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
docker compose config
docker compose build
```
