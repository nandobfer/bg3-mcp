# Use Quando Integrar Fontes HTTP

## Antes de implementar

Verifique e documente:

- Endpoints e metodos aceitos.
- Autenticacao e armazenamento de credenciais.
- Rate limits e politica de uso automatizado.
- Licenca e obrigacoes de atribuicao.
- Paginacao, IDs estaveis e formatos de erro.
- Timeouts, limites de tamanho e comportamento de retry.

## Cliente

- Use uma URL-base configuravel.
- Reutilize conexoes por meio de um cliente compartilhado.
- Envie `User-Agent` identificavel.
- Aplique timeout e limite de tamanho de resposta.
- Limite concorrencia por fonte.
- Trate status HTTP antes de desserializar sucesso.
- Desserialize para tipos locais; nao espalhe JSON generico pelo dominio.

## Retry e backoff

Retries sao permitidos apenas para operacoes idempotentes e falhas transitorias,
como timeout de conexao, `429` e alguns `5xx`. Use backoff limitado e respeite
`Retry-After` quando presente. Erros de validacao, autenticacao ou `404` nao
devem ser repetidos automaticamente.

## Conteudo nao confiavel

- Sanitize snippets e HTML antes de retornar texto.
- Limite tamanho antes e depois da transformacao.
- Nao siga URLs arbitrarias vindas do cliente MCP.
- Valide redirects para a origem esperada quando o cliente HTTP os seguir.

## Testes

Use um servidor mockado para cobrir sucesso, resposta malformada, timeout,
`429`, `5xx`, `404`, resposta grande e retry. Testes externos reais devem ser
opt-in.
