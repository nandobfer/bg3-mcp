FROM rust:1.89-bookworm AS builder

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY src ./src

RUN cargo build --locked --release

FROM debian:bookworm-slim AS runtime

RUN apt-get update \
    && apt-get install --yes --no-install-recommends ca-certificates curl \
    && rm -rf /var/lib/apt/lists/* \
    && groupadd --system --gid 10001 bg3mcp \
    && useradd --system --uid 10001 --gid bg3mcp \
        --home-dir /nonexistent --shell /usr/sbin/nologin bg3mcp

WORKDIR /app

COPY --from=builder /app/target/release/bg3-mcp /usr/local/bin/bg3-mcp

ENV BG3_MCP_HOST=0.0.0.0
ENV BG3_MCP_PORT=3000

USER bg3mcp

EXPOSE 3000

ENTRYPOINT ["/usr/local/bin/bg3-mcp"]
