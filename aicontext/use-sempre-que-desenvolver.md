# Use Sempre Que Desenvolver

## Principios

- Comece com a menor implementacao correta.
- Crie modulos apenas quando houver codigo suficiente para justificar a
  separacao.
- Prefira tipos de dominio a mapas JSON sem contrato.
- Mantenha handlers MCP finos; regras pertencem aos servicos de dominio.
- Mantenha clientes externos isolados das representacoes do protocolo MCP.
- Nao esconda estado pendente: use `TBD` em vez de inventar contratos.

## Estrutura atual

```text
src/
  main.rs
  lib.rs
  server.rs
  config.rs
  error.rs
  mcp.rs
  wiki/
  infrastructure/
```

O dominio de mods ainda nao possui codigo. Crie modulos novos somente quando
houver implementacao concreta.

## Rust

- Use `Result` e erros tipados em operacoes faliveis.
- Nao exponha erros internos ou respostas brutas de terceiros ao cliente MCP.
- Use `serde` para contratos serializaveis e valide valores alem do shape.
- Mantenha funcoes assincronas cancelaveis e limitadas por timeout.
- Use `tracing` para logs estruturados.
- Evite clones e alocacoes sem necessidade, sem sacrificar clareza.

## Testes

- Teste comportamento e contratos publicos.
- Use servidor HTTP mockado para Action API, REST, `429`, `5xx` e timeouts.
- Cubra redirecionamentos com e sem fragmentos.
- Cubra limites de pagina e quantidade, inputs invalidos e a ausencia deliberada
  de limite de bytes para bodies e conteudo.
- Separe testes externos manuais dos testes automatizados.

## Seguranca e privacidade

- Trate todo input MCP e toda resposta externa como nao confiavel.
- Remova ou sanitize HTML antes de expor texto ao cliente.
- Nao permita URLs arbitrarias fornecidas pelo cliente quando a ferramenta
  deveria consultar apenas uma fonte configurada.
- Nao grave secrets, headers de autorizacao ou conteudo integral em logs.

## Checklist de encerramento

1. O input e validado e respeita os limites de pagina e quantidade aplicaveis.
2. Falhas externas sao normalizadas.
3. A resposta inclui fonte e atribuicao quando aplicavel.
4. Testes foram adicionados ou a ausencia foi justificada.
5. A documentacao contextual foi atualizada quando o contrato mudou.
6. Novas variaveis constam no `.env.example`.
