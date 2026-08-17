use std::{
    collections::HashMap,
    net::{IpAddr, SocketAddr},
    sync::Arc,
    time::{Duration, Instant},
};

use axum::{
    Json, Router,
    extract::{ConnectInfo, Request, State},
    http::{StatusCode, header},
    middleware::{Next, from_fn_with_state},
    response::{IntoResponse, Response},
    routing::get,
};
use rmcp::transport::streamable_http_server::{
    StreamableHttpServerConfig, StreamableHttpService, session::local::LocalSessionManager,
};
use serde::Serialize;
use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;
use tower_http::cors::CorsLayer;
use tracing::info;

use crate::{config::Config, error::AppError, mcp::WikiMcpServer, wiki::WikiService};

pub async fn run(config: Config) -> Result<(), AppError> {
    let cancellation = CancellationToken::new();
    let router = build_router(&config, cancellation.child_token())?;
    let address = config.bind_address();
    let listener = tokio::net::TcpListener::bind(&address)
        .await
        .map_err(AppError::Bind)?;
    info!(%address, "BG3 MCP server listening");

    let shutdown = cancellation.clone();
    axum::serve(
        listener,
        router.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .with_graceful_shutdown(async move {
        if tokio::signal::ctrl_c().await.is_ok() {
            shutdown.cancel();
        }
    })
    .await
    .map_err(AppError::Serve)
}

pub fn build_router(config: &Config, cancellation: CancellationToken) -> Result<Router, AppError> {
    let wiki_service = WikiService::from_config(config)?;
    let service_factory = move || Ok(WikiMcpServer::new(wiki_service.clone()));
    let mcp_config = StreamableHttpServerConfig::default()
        .with_legacy_session_mode(false)
        .with_json_response(true)
        .disable_allowed_hosts()
        .disable_allowed_origins()
        .with_cancellation_token(cancellation);
    let mcp_service = StreamableHttpService::new(
        service_factory,
        LocalSessionManager::default().into(),
        mcp_config,
    );
    let rate_limiter = Arc::new(RateLimiter::new(config.rate_limit_per_minute));

    let mcp_router = Router::new()
        .nest_service("/mcp", mcp_service)
        .layer(from_fn_with_state(rate_limiter, rate_limit));

    Ok(Router::new()
        .route("/health", get(health))
        .merge(mcp_router)
        .layer(CorsLayer::permissive()))
}

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
    service: &'static str,
    version: &'static str,
}

async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok",
        service: "bg3-mcp",
        version: env!("CARGO_PKG_VERSION"),
    })
}

struct RateLimiter {
    max_requests: u32,
    clients: Mutex<HashMap<IpAddr, RateWindow>>,
}

impl RateLimiter {
    fn new(max_requests: u32) -> Self {
        Self {
            max_requests,
            clients: Mutex::new(HashMap::new()),
        }
    }

    async fn check(&self, ip: IpAddr) -> Result<(), u64> {
        let now = Instant::now();
        let mut clients = self.clients.lock().await;
        clients.retain(|_, window| now.duration_since(window.started) < Duration::from_secs(60));
        let window = clients.entry(ip).or_insert(RateWindow {
            started: now,
            count: 0,
        });
        if now.duration_since(window.started) >= Duration::from_secs(60) {
            window.started = now;
            window.count = 0;
        }
        if window.count >= self.max_requests {
            let elapsed = now.duration_since(window.started).as_secs();
            return Err(60_u64.saturating_sub(elapsed).max(1));
        }
        window.count += 1;
        Ok(())
    }
}

struct RateWindow {
    started: Instant,
    count: u32,
}

async fn rate_limit(
    State(limiter): State<Arc<RateLimiter>>,
    request: Request,
    next: Next,
) -> Response {
    let ip = request
        .extensions()
        .get::<ConnectInfo<SocketAddr>>()
        .map(|ConnectInfo(address)| address.ip())
        .unwrap_or(IpAddr::V4(std::net::Ipv4Addr::UNSPECIFIED));

    match limiter.check(ip).await {
        Ok(()) => next.run(request).await,
        Err(retry_after) => (
            StatusCode::TOO_MANY_REQUESTS,
            [(header::RETRY_AFTER, retry_after.to_string())],
            "rate limit exceeded",
        )
            .into_response(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn rate_limiter_rejects_requests_over_limit() {
        let limiter = RateLimiter::new(1);
        let ip = IpAddr::V4(std::net::Ipv4Addr::LOCALHOST);

        assert!(limiter.check(ip).await.is_ok());
        assert!(limiter.check(ip).await.is_err());
    }
}
