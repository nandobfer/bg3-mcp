# Use Para Atualizar Containers

## Objetivo

O deploy inicial possui apenas o servico `bg3-mcp`. Nao adicione banco, Redis ou
outros containers sem necessidade concreta.

## Dockerfile

- Use build multi-stage.
- Fixe uma versao suportada da imagem Rust.
- Compile com `cargo build --locked --release`.
- Nao leve Cargo, rustc ou fontes para a imagem final.
- Instale certificados CA para HTTPS.
- Execute como usuario sem privilegios.
- Mantenha o comando final como o binario `bg3-mcp`.

O build usa `Cargo.toml`, `Cargo.lock`, `src/` e o binario `bg3-mcp` existentes.

## Compose

- Carregue configuracao de `.env`.
- Publique somente a porta MCP.
- Use `restart: unless-stopped`.
- Verifique `/health` sem consultar fontes externas.
- Nao declare redes externas sem dependencia real.
- Nao fixe `container_name` se isso impedir multiplas instancias; o Compose atual
  nao precisa dele.

## Ambiente

- `.env.example` e o contrato versionado.
- `.env` e local e deve permanecer no `.gitignore`.
- Placeholders devem indicar claramente o que o operador precisa alterar.
- Novas variaveis precisam ser documentadas em
  `use-diretrizes-do-projeto.md`.

## Validacao

1. `docker compose config`
2. `docker compose build`
3. `docker compose up -d`
4. Verificar o estado do health check.
5. Fazer uma chamada MCP de smoke test.

As cinco etapas fazem parte da verificacao normal do projeto.
