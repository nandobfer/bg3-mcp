use std::{env, net::IpAddr, time::Duration};

use thiserror::Error;
use url::Url;

#[derive(Clone, Debug)]
pub struct Config {
    pub host: IpAddr,
    pub port: u16,
    pub transport: String,
    pub log_filter: String,
    pub wiki_base_url: Url,
    pub user_agent: String,
    pub http_timeout: Duration,
    pub max_concurrency: usize,
    pub cache_ttl: Duration,
    pub cache_max_entries: u64,
    pub http_retry_max: usize,
    pub rate_limit_per_minute: u32,
}

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("environment variable {name} must be set")]
    Missing { name: &'static str },
    #[error("invalid value for {name}: {message}")]
    Invalid { name: &'static str, message: String },
}

impl Config {
    pub fn from_env() -> Result<Self, ConfigError> {
        let host = parse("BG3_MCP_HOST", "0.0.0.0")?;
        let port = parse("BG3_MCP_PORT", "3000")?;
        let transport = value("BG3_MCP_TRANSPORT", "streamable-http");
        if transport != "streamable-http" {
            return Err(ConfigError::Invalid {
                name: "BG3_MCP_TRANSPORT",
                message: "only streamable-http is supported".to_string(),
            });
        }

        let wiki_base_url =
            Url::parse(&value("BG3_WIKI_BASE_URL", "https://bg3.wiki")).map_err(|error| {
                ConfigError::Invalid {
                    name: "BG3_WIKI_BASE_URL",
                    message: error.to_string(),
                }
            })?;
        if !matches!(wiki_base_url.scheme(), "http" | "https") {
            return Err(ConfigError::Invalid {
                name: "BG3_WIKI_BASE_URL",
                message: "scheme must be http or https".to_string(),
            });
        }

        let user_agent = required("BG3_MCP_USER_AGENT")?;
        let timeout_secs = nonzero_parse("BG3_MCP_HTTP_TIMEOUT_SECS", "15")?;
        let max_concurrency = nonzero_parse("BG3_MCP_MAX_CONCURRENCY", "1")?;
        let cache_ttl_secs = nonzero_parse("BG3_MCP_CACHE_TTL_SECS", "300")?;
        let cache_max_entries = nonzero_parse("BG3_MCP_CACHE_MAX_ENTRIES", "512")?;
        let http_retry_max = parse("BG3_MCP_HTTP_RETRY_MAX", "2")?;
        let rate_limit_per_minute = nonzero_parse("BG3_MCP_RATE_LIMIT_PER_MINUTE", "60")?;

        Ok(Self {
            host,
            port,
            transport,
            log_filter: value("BG3_MCP_LOG", "info"),
            wiki_base_url,
            user_agent,
            http_timeout: Duration::from_secs(timeout_secs),
            max_concurrency,
            cache_ttl: Duration::from_secs(cache_ttl_secs),
            cache_max_entries,
            http_retry_max,
            rate_limit_per_minute,
        })
    }

    pub fn bind_address(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}

fn value(name: &'static str, default: &str) -> String {
    env::var(name).unwrap_or_else(|_| default.to_string())
}

fn required(name: &'static str) -> Result<String, ConfigError> {
    let value = env::var(name).map_err(|_| ConfigError::Missing { name })?;
    if value.trim().is_empty() {
        return Err(ConfigError::Invalid {
            name,
            message: "value cannot be empty".to_string(),
        });
    }
    Ok(value)
}

fn parse<T>(name: &'static str, default: &str) -> Result<T, ConfigError>
where
    T: std::str::FromStr,
    T::Err: std::fmt::Display,
{
    value(name, default)
        .parse::<T>()
        .map_err(|error| ConfigError::Invalid {
            name,
            message: error.to_string(),
        })
}

fn nonzero_parse<T>(name: &'static str, default: &str) -> Result<T, ConfigError>
where
    T: std::str::FromStr + PartialEq + Default,
    T::Err: std::fmt::Display,
{
    let parsed = parse(name, default)?;
    if parsed == T::default() {
        return Err(ConfigError::Invalid {
            name,
            message: "value must be greater than zero".to_string(),
        });
    }
    Ok(parsed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bind_address_joins_host_and_port() {
        let config = Config {
            host: "0.0.0.0".parse().unwrap(),
            port: 4000,
            transport: "streamable-http".to_string(),
            log_filter: "info".to_string(),
            wiki_base_url: Url::parse("https://bg3.wiki").unwrap(),
            user_agent: "test".to_string(),
            http_timeout: Duration::from_secs(1),
            max_concurrency: 1,
            cache_ttl: Duration::from_secs(1),
            cache_max_entries: 1,
            http_retry_max: 0,
            rate_limit_per_minute: 1,
        };

        assert_eq!(config.bind_address(), "0.0.0.0:4000");
    }
}
