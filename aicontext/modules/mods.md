# Modulo: Mods

## Estado

**TBD**. Nenhum provedor foi escolhido e nenhuma ferramenta de mods deve ser
tratada como contrato implementado.

## Objetivo preliminar

Permitir pesquisa e navegacao de mods de Baldur's Gate 3 sem instalar arquivos,
alterar load order ou modificar uma instalacao local do jogo.

## Informacoes necessarias

Antes de implementar, definir e verificar:

- Site, API ou catalogo de origem.
- Documentacao e estabilidade da API.
- Autenticacao e armazenamento de credenciais.
- Rate limits e regras de automacao.
- Licenca e condicoes de redistribuicao.
- Paginacao, pesquisa e filtros.
- IDs estaveis de jogos, mods, arquivos e versoes.
- Compatibilidade, dependencias e requisitos.
- Disponibilidade de changelogs, imagens e metricas.
- Se o escopo inclui apenas consulta ou tambem download.

## Ferramentas candidatas

- `mods_search`
- `mods_get`
- `mods_list_files`
- `mods_get_requirements`

Nomes, schemas e autenticacao permanecem pendentes.

## Regras

- Nao reutilize modelos da wiki como modelo de mods.
- Nao implemente download ou instalacao sem requisito e permissao explicitos.
- Credenciais devem vir de ambiente ou secret.
- Realize testes equivalentes aos da wiki antes de criar o cliente definitivo.
