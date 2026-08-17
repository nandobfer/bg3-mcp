use std::{net::IpAddr, time::Duration};

use bg3_mcp::Config;
use url::Url;

pub fn test_config(base_url: &str) -> Config {
    Config {
        host: IpAddr::V4(std::net::Ipv4Addr::LOCALHOST),
        port: 3000,
        transport: "streamable-http".to_string(),
        log_filter: "info".to_string(),
        wiki_base_url: Url::parse(base_url).unwrap(),
        user_agent: "BG3MCP-Test/0.1".to_string(),
        http_timeout: Duration::from_secs(2),
        max_concurrency: 1,
        cache_ttl: Duration::from_secs(60),
        cache_max_entries: 32,
        http_retry_max: 0,
        rate_limit_per_minute: 60,
    }
}
