use std::{
    collections::VecDeque,
    sync::Arc,
    time::{Duration, Instant},
};

use moka::future::Cache;
use reqwest::{Client, StatusCode, header::RETRY_AFTER};
use serde::de::DeserializeOwned;
use serde_json::Value;
use tokio::sync::{Mutex, Semaphore};
use tracing::{debug, warn};
use url::Url;

use crate::{Config, error::ModIoError};

use super::models::{ListApiResponse, ModApi, ModPlatform};

const MAX_RETRY_AFTER: Duration = Duration::from_secs(60);

#[derive(Clone)]
pub struct ModIoClient {
    client: Client,
    mods_url: Url,
    api_key: String,
    semaphore: Arc<Semaphore>,
    cache: Cache<String, Arc<Value>>,
    retry_max: usize,
    rate_limiter: Arc<UpstreamRateLimiter>,
    cooldown_until: Arc<Mutex<Option<Instant>>>,
}

impl ModIoClient {
    pub fn new(config: &Config) -> Result<Self, crate::error::AppError> {
        let client = Client::builder()
            .user_agent(&config.user_agent)
            .timeout(config.http_timeout)
            .redirect(reqwest::redirect::Policy::none())
            .build()?;
        let mut base_url = config.modio_base_url.clone();
        if !base_url.path().ends_with('/') {
            let path = format!("{}/", base_url.path());
            base_url.set_path(&path);
        }
        let mods_url = base_url.join(&format!("games/{}/mods", config.modio_game_id))?;

        Ok(Self {
            client,
            mods_url,
            api_key: config.modio_api_key.clone(),
            semaphore: Arc::new(Semaphore::new(config.max_concurrency)),
            cache: Cache::builder()
                .max_capacity(config.cache_max_entries)
                .time_to_live(config.cache_ttl)
                .build(),
            retry_max: config.http_retry_max,
            rate_limiter: Arc::new(UpstreamRateLimiter::new(config.modio_rate_limit_per_minute)),
            cooldown_until: Arc::new(Mutex::new(None)),
        })
    }

    pub async fn search(
        &self,
        query: Option<&str>,
        platform: ModPlatform,
        sort: &str,
        limit: u32,
        cursor: u64,
    ) -> Result<ListApiResponse<ModApi>, ModIoError> {
        let mut params = vec![
            ("_limit", limit.to_string()),
            ("_offset", cursor.to_string()),
            ("_sort", sort.to_string()),
        ];
        if let Some(query) = query {
            params.push(("_q", query.to_string()));
        }
        self.get(self.mods_url.clone(), platform, params).await
    }

    pub async fn get_mod(&self, mod_id: u64, platform: ModPlatform) -> Result<ModApi, ModIoError> {
        let url = self
            .mods_url
            .join(&format!("mods/{mod_id}"))
            .map_err(|_| ModIoError::InvalidInput("mod_id cannot form a URL".to_string()))?;
        self.get(url, platform, Vec::new()).await
    }

    async fn get<T>(
        &self,
        url: Url,
        platform: ModPlatform,
        mut params: Vec<(&str, String)>,
    ) -> Result<T, ModIoError>
    where
        T: DeserializeOwned,
    {
        params.sort();
        let cache_key = format!("{}:{}:{params:?}", url.path(), platform.as_str());
        if let Some(value) = self.cache.get(&cache_key).await {
            debug!(cache = "hit", endpoint = %url.path(), "mod.io request");
            return deserialize((*value).clone());
        }

        for attempt in 0..=self.retry_max {
            match self.get_once(url.clone(), platform, &params).await {
                Ok(value) => {
                    self.cache
                        .insert(cache_key.clone(), Arc::new(value.clone()))
                        .await;
                    return deserialize(value);
                }
                Err(RequestFailure::Retryable { retry_after }) if attempt < self.retry_max => {
                    self.wait_before_retry(attempt, retry_after).await;
                }
                Err(failure) => return Err(failure.into_modio_error()),
            }
        }
        Err(ModIoError::Unavailable)
    }

    async fn get_once(
        &self,
        mut url: Url,
        platform: ModPlatform,
        params: &[(&str, String)],
    ) -> Result<Value, RequestFailure> {
        let _permit = self
            .semaphore
            .acquire()
            .await
            .map_err(|_| RequestFailure::Unavailable)?;
        self.wait_for_cooldown().await;
        self.rate_limiter.acquire().await;

        {
            let mut query = url.query_pairs_mut();
            for (name, value) in params {
                query.append_pair(name, value);
            }
            query.append_pair("api_key", &self.api_key);
        }
        let response = self
            .client
            .get(url)
            .header("X-Modio-Platform", platform.as_str())
            .header("Accept", "application/json")
            .send()
            .await
            .map_err(RequestFailure::from_reqwest)?;
        let status = response.status();
        let retry_after = parse_retry_after(response.headers().get(RETRY_AFTER));
        if status == StatusCode::TOO_MANY_REQUESTS {
            let delay = retry_after.unwrap_or(MAX_RETRY_AFTER);
            *self.cooldown_until.lock().await = Instant::now().checked_add(delay);
            return Err(RequestFailure::Retryable {
                retry_after: Some(delay),
            });
        }
        if status.is_server_error() {
            return Err(RequestFailure::Retryable { retry_after });
        }
        if status == StatusCode::NOT_FOUND {
            return Err(RequestFailure::NotFound);
        }
        if matches!(status, StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN) {
            return Err(RequestFailure::Unauthorized);
        }
        if !status.is_success() {
            return Err(RequestFailure::Rejected);
        }
        let value = response
            .json::<Value>()
            .await
            .map_err(|_| RequestFailure::UnexpectedResponse)?;
        if value.get("error").is_some() {
            return Err(RequestFailure::Rejected);
        }
        Ok(value)
    }

    async fn wait_for_cooldown(&self) {
        loop {
            let delay = self
                .cooldown_until
                .lock()
                .await
                .and_then(|deadline| deadline.checked_duration_since(Instant::now()));
            match delay {
                Some(delay) if !delay.is_zero() => tokio::time::sleep(delay).await,
                _ => return,
            }
        }
    }

    async fn wait_before_retry(&self, attempt: usize, retry_after: Option<Duration>) {
        let exponential_ms = 250_u64.saturating_mul(1_u64 << attempt.min(6));
        let jitter_ms = fastrand::u64(0..=100);
        let delay =
            retry_after.unwrap_or_else(|| Duration::from_millis(exponential_ms + jitter_ms));
        warn!(attempt = attempt + 1, ?delay, "retrying mod.io request");
        tokio::time::sleep(delay).await;
    }
}

fn deserialize<T: DeserializeOwned>(value: Value) -> Result<T, ModIoError> {
    serde_json::from_value(value).map_err(|_| ModIoError::UnexpectedResponse)
}

fn parse_retry_after(value: Option<&reqwest::header::HeaderValue>) -> Option<Duration> {
    value
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.parse::<u64>().ok())
        .map(|seconds| Duration::from_secs(if seconds == 0 { 60 } else { seconds }))
        .map(|duration| duration.min(MAX_RETRY_AFTER))
}

struct UpstreamRateLimiter {
    max_requests: usize,
    requests: Mutex<VecDeque<Instant>>,
}

impl UpstreamRateLimiter {
    fn new(max_requests: u32) -> Self {
        Self {
            max_requests: max_requests as usize,
            requests: Mutex::new(VecDeque::new()),
        }
    }

    async fn acquire(&self) {
        loop {
            let mut requests = self.requests.lock().await;
            let now = Instant::now();
            while requests
                .front()
                .is_some_and(|time| now.duration_since(*time) >= Duration::from_secs(60))
            {
                requests.pop_front();
            }
            if requests.len() < self.max_requests {
                requests.push_back(now);
                return;
            }
            let delay = requests
                .front()
                .map(|time| Duration::from_secs(60).saturating_sub(now.duration_since(*time)))
                .unwrap_or_default();
            drop(requests);
            tokio::time::sleep(delay).await;
        }
    }
}

enum RequestFailure {
    Timeout,
    Retryable { retry_after: Option<Duration> },
    NotFound,
    Unauthorized,
    Rejected,
    UnexpectedResponse,
    Unavailable,
}

impl RequestFailure {
    fn from_reqwest(error: reqwest::Error) -> Self {
        if error.is_timeout() {
            Self::Timeout
        } else if error.is_connect() {
            Self::Retryable { retry_after: None }
        } else {
            Self::Unavailable
        }
    }

    fn into_modio_error(self) -> ModIoError {
        match self {
            Self::Timeout => ModIoError::Timeout,
            Self::Retryable { .. } | Self::Unavailable => ModIoError::Unavailable,
            Self::NotFound => ModIoError::NotFound,
            Self::Unauthorized => ModIoError::Unauthorized,
            Self::Rejected => ModIoError::Rejected,
            Self::UnexpectedResponse => ModIoError::UnexpectedResponse,
        }
    }
}

#[cfg(test)]
mod tests {
    use reqwest::header::HeaderValue;

    use super::*;

    #[test]
    fn retry_after_is_bounded_and_zero_uses_rolling_limit_fallback() {
        assert_eq!(
            parse_retry_after(Some(&HeaderValue::from_static("999999999999"))),
            Some(MAX_RETRY_AFTER)
        );
        assert_eq!(
            parse_retry_after(Some(&HeaderValue::from_static("0"))),
            Some(MAX_RETRY_AFTER)
        );
    }
}
