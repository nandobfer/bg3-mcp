# Modulo: Mods

## Estado

**Implementado** com a REST API v1 do mod.io em modo somente leitura.

## Fonte verificada

- Catalogo: `https://mod.io/g/baldursgate3`.
- API path: fornecido por conta pelo painel do mod.io e configuravel por ambiente.
- Game ID: `6715`, verificado pelo filtro `name_id=baldursgate3`.
- Autenticacao: API key de 32 caracteres no query parameter `api_key`.
- Limite documentado para chaves de usuario: 60 requisicoes por minuto.
- Plataformas do jogo: `windows`, `mac`, `xboxseriesx` e `ps5`.
- Documentacao: `https://docs.mod.io/restapiref/`.
- Termos: `https://mod.io/terms`.

## Ferramentas

### `mods_search`

Lista mods quando `query` e omitido e usa `_q` quando ele e informado. Aceita:

- `query`: opcional, nao vazio e com ate 200 caracteres.
- `platform`: `windows`, `mac`, `ps5` ou `xboxseriesx`; default `windows`.
- `sort`: `updated`, `newest`, `downloads`, `popular`, `rating` ou `name`;
  default `updated`.
- `limit`: default 10, intervalo de 1 a 20.
- `cursor`: offset de continuacao, de 0 a 100000.

A resposta inclui resumos, total, proximo cursor e atribuicao.

### `mods_get`

Recebe `mod_id` numerico positivo e plataforma. Retorna descricao plaintext,
autoria, tags, plataformas, midia, estatisticas, opcoes de maturidade e credito e
o modfile atual. URLs de download nao fazem parte do contrato.

## Comportamento

- O header `X-Modio-Platform` e enviado em todas as requisicoes.
- `_offset` e usado como cursor; o proximo cursor so existe quando ha resultados
  restantes.
- Apenas respostas bem-sucedidas entram no cache.
- Textos e URLs retornados sao dados externos nao confiaveis.
- Cada resposta inclui o catalogo do BG3 no mod.io e a URL dos termos.
- Bitmasks de maturidade e credito sao convertidos em nomes explicitos.

## Fora do escopo

- Download de arquivos e exposicao de `download.binary_url`.
- Instalacao, assinatura, rating, autenticacao de usuario e alteracao de load
  order.
- Listagem historica de modfiles e consulta detalhada de dependencias.
- Crawling ou espelhamento do catalogo.
