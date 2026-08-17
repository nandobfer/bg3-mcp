# AGENTS.md

Guia normativo para agentes de IA e colaboradores que trabalhem neste
repositorio.

## 1. Ordem obrigatoria de leitura

Antes de implementar, leia nesta ordem:

1. `aicontext/README.md`
2. `aicontext/use-sempre-que-desenvolver.md`
3. O guia relacionado a tarefa:
   - MCP: `aicontext/use-quando-desenvolver-mcp.md`
   - Integracoes HTTP: `aicontext/use-quando-integrar-fontes-http.md`
   - Docker: `aicontext/use-para-atualizar-containers.md`
   - Arquitetura: `aicontext/use-diretrizes-do-projeto.md`
4. A documentacao do modulo afetado em `aicontext/modules/`.

## 2. Estado atual

O servidor MCP e a integracao de leitura com a bg3.wiki estao implementados. O
dominio de mods continua pendente. Preserve nos documentos a distincao entre
`implementado`, `verificado`, `planejado` e `TBD`.

## 3. Regras nao negociaveis

- Use Rust idiomatico e tipos explicitos nos limites publicos.
- Nao use `unwrap` ou `expect` em caminhos que processam entrada ou falhas
  externas, salvo em testes ou invariantes justificadas.
- Mantenha wiki e mods como dominios separados. Nao crie uma abstracao comum
  sem reutilizacao concreta.
- Valide todos os inputs MCP e imponha limites de pagina e quantidade. Por
  decisao do projeto, nao imponha limite de bytes ao body MCP nem ao conteudo
  baixado ou retornado pela wiki.
- Toda resposta baseada em fonte externa deve incluir atribuicao e URL.
- Nunca registre tokens, credenciais, corpos sensiveis ou conteudo integral sem
  necessidade operacional.
- Nunca adicione credenciais ao repositorio ou a imagem Docker.
- A bg3.wiki deve ser consultada sob demanda. Nao implemente crawling,
  espelhamento ou coleta em massa.
- Testes automatizados nao devem depender da disponibilidade da bg3.wiki.
- Alteracoes de contrato, arquitetura ou comportamento devem atualizar o arquivo
  correspondente em `aicontext/`.

## 4. Integracoes externas

- Use um `User-Agent` identificavel, configurado por ambiente.
- Configure timeout, limite de concorrencia, cache e backoff.
- Trate `429`, `5xx`, timeout e indisponibilidade como falhas externas
  normalizadas.
- Respeite licencas, atribuicao e politicas da fonte.
- Use URL-base configuravel para permitir testes com servidor mockado.
- Na Action API da bg3.wiki, envie `POST` como
  `application/x-www-form-urlencoded`, nunca como JSON.

## 5. Containers e configuracao

- O container final deve executar como usuario sem privilegios.
- Use build multi-stage e nao inclua o toolchain Rust na imagem final.
- Mantenha apenas um servico no Compose ate surgir uma necessidade concreta.
- Novas variaveis devem ser documentadas, adicionadas ao `.env.example` e
  propagadas no Compose quando necessario.
- `.env` e local. Nunca substitua placeholders por segredos versionados.

## 6. Verificacao minima

Execute conforme o escopo:

1. `cargo fmt --check`
2. `cargo clippy --all-targets --all-features -- -D warnings`
3. `cargo test`
4. `docker compose config`, quando Docker ou ambiente forem alterados
5. `docker compose build`, quando houver uma aplicacao compilavel

Testes externos reais devem ser manuais ou explicitamente marcados como
integracao externa.

## 7. Documentacao

- `README.md` e destinado a usuarios finais.
- Detalhes tecnicos, investigacoes e decisoes pertencem a `aicontext/`.
- Um novo dominio deve receber `aicontext/modules/<dominio>.md`.
- Evite duplicar a mesma regra em varios arquivos; prefira referencias.
