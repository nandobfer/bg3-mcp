use std::{sync::Arc, time::Duration};

use moka::future::Cache;
use reqwest::{Client, StatusCode, header::RETRY_AFTER};
use serde_json::Value;
use tokio::sync::Semaphore;
use tracing::{debug, warn};
use url::Url;

use crate::{
    config::Config,
    error::{AppError, WikiError},
};

#[derive(Clone)]
pub struct MediaWikiHttpClient {
    client: Client,
    action_url: Url,
    rest_page_url: Url,
    semaphore: Arc<Semaphore>,
    cache: Cache<String, Arc<Value>>,
    retry_max: usize,
}

impl MediaWikiHttpClient {
    pub fn new(config: &Config) -> Result<Self, AppError> {
        let client = Client::builder()
            .user_agent(&config.user_agent)
            .timeout(config.http_timeout)
            .redirect(reqwest::redirect::Policy::limited(5))
            .build()?;
        let action_url = config.wiki_base_url.join("/w/api.php")?;
        let rest_page_url = config.wiki_base_url.join("/w/rest.php/v1/page/")?;
        let cache = Cache::builder()
            .max_capacity(config.cache_max_entries)
            .time_to_live(config.cache_ttl)
            .build();

        Ok(Self {
            client,
            action_url,
            rest_page_url,
            semaphore: Arc::new(Semaphore::new(config.max_concurrency)),
            cache,
            retry_max: config.http_retry_max,
        })
    }

    pub async fn action(&self, mut params: Vec<(&str, String)>) -> Result<Value, WikiError> {
        params.push(("format", "json".to_string()));
        params.push(("formatversion", "2".to_string()));
        params.push(("maxlag", "5".to_string()));
        params.sort();
        let cache_key = format!("action:{params:?}");
        if let Some(value) = self.cache.get(&cache_key).await {
            debug!(cache = "hit", endpoint = "action", "wiki request");
            return Ok((*value).clone());
        }

        for attempt in 0..=self.retry_max {
            let result = self.action_once(&params).await;
            match result {
                Ok(value) => {
                    self.cache
                        .insert(cache_key.clone(), Arc::new(value.clone()))
                        .await;
                    return Ok(value);
                }
                Err(RequestFailure::Retryable { retry_after }) if attempt < self.retry_max => {
                    self.wait_before_retry(attempt, retry_after).await;
                }
                Err(failure) => return Err(failure.into_wiki_error()),
            }
        }

        Err(WikiError::Unavailable)
    }

    pub async fn rest_page(&self, encoded_title: &str) -> Result<Value, WikiError> {
        let url = self
            .rest_page_url
            .join(encoded_title)
            .map_err(|_| WikiError::InvalidInput("page title cannot form a URL".to_string()))?;
        let cache_key = format!("rest:{url}");
        if let Some(value) = self.cache.get(&cache_key).await {
            debug!(cache = "hit", endpoint = "rest", "wiki request");
            return Ok((*value).clone());
        }

        for attempt in 0..=self.retry_max {
            let result = self.get_once(url.clone()).await;
            match result {
                Ok(value) => {
                    self.cache
                        .insert(cache_key.clone(), Arc::new(value.clone()))
                        .await;
                    return Ok(value);
                }
                Err(RequestFailure::Retryable { retry_after }) if attempt < self.retry_max => {
                    self.wait_before_retry(attempt, retry_after).await;
                }
                Err(failure) => return Err(failure.into_wiki_error()),
            }
        }

        Err(WikiError::Unavailable)
    }

    async fn action_once(&self, params: &[(&str, String)]) -> Result<Value, RequestFailure> {
        let _permit = self
            .semaphore
            .acquire()
            .await
            .map_err(|_| RequestFailure::Unavailable)?;
        let response = self
            .client
            .post(self.action_url.clone())
            .form(params)
            .send()
            .await
            .map_err(RequestFailure::from_reqwest)?;
        let status = response.status();
        let retry_after = parse_retry_after(response.headers().get(RETRY_AFTER));
        if status == StatusCode::TOO_MANY_REQUESTS || status.is_server_error() {
            return Err(RequestFailure::Retryable { retry_after });
        }
        if !status.is_success() {
            return Err(RequestFailure::Status(status));
        }

        let value = response
            .json::<Value>()
            .await
            .map_err(|_| RequestFailure::UnexpectedResponse)?;
        if let Some(error) = value.get("error") {
            let code = error
                .get("code")
                .and_then(Value::as_str)
                .unwrap_or("unknown");
            if code == "maxlag" {
                return Err(RequestFailure::Retryable { retry_after });
            }
            return Err(RequestFailure::Api(code.to_string()));
        }
        Ok(value)
    }

    async fn get_once(&self, url: Url) -> Result<Value, RequestFailure> {
        let _permit = self
            .semaphore
            .acquire()
            .await
            .map_err(|_| RequestFailure::Unavailable)?;
        let response = self
            .client
            .get(url)
            .send()
            .await
            .map_err(RequestFailure::from_reqwest)?;
        let status = response.status();
        let retry_after = parse_retry_after(response.headers().get(RETRY_AFTER));
        if status == StatusCode::TOO_MANY_REQUESTS || status.is_server_error() {
            return Err(RequestFailure::Retryable { retry_after });
        }
        if status == StatusCode::NOT_FOUND {
            return Err(RequestFailure::NotFound);
        }
        if !status.is_success() {
            return Err(RequestFailure::Status(status));
        }
        response
            .json::<Value>()
            .await
            .map_err(|_| RequestFailure::UnexpectedResponse)
    }

    async fn wait_before_retry(&self, attempt: usize, retry_after: Option<Duration>) {
        let exponential_ms = 250_u64.saturating_mul(1_u64 << attempt.min(6));
        let jitter_ms = fastrand::u64(0..=100);
        let delay = retry_after
            .map(|duration| duration.min(Duration::from_secs(30)))
            .unwrap_or_else(|| Duration::from_millis(exponential_ms + jitter_ms));
        warn!(attempt = attempt + 1, ?delay, "retrying bg3.wiki request");
        tokio::time::sleep(delay).await;
    }
}

fn parse_retry_after(value: Option<&reqwest::header::HeaderValue>) -> Option<Duration> {
    value
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.parse::<u64>().ok())
        .map(Duration::from_secs)
}

enum RequestFailure {
    Timeout,
    Retryable { retry_after: Option<Duration> },
    Status(StatusCode),
    Api(String),
    NotFound,
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

    fn into_wiki_error(self) -> WikiError {
        match self {
            Self::Timeout => WikiError::Timeout,
            Self::Retryable { .. } | Self::Unavailable => WikiError::Unavailable,
            Self::Status(status) => WikiError::Rejected(status.to_string()),
            Self::Api(code) => WikiError::Rejected(code),
            Self::NotFound => WikiError::NotFound("page".to_string()),
            Self::UnexpectedResponse => WikiError::UnexpectedResponse,
        }
    }
}
