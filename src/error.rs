use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("failed to build HTTP client: {0}")]
    HttpClient(#[from] reqwest::Error),
    #[error("failed to bind server: {0}")]
    Bind(#[source] std::io::Error),
    #[error("server failed: {0}")]
    Serve(#[source] std::io::Error),
    #[error("invalid wiki endpoint: {0}")]
    InvalidEndpoint(#[from] url::ParseError),
}

#[derive(Debug, Error, Clone)]
pub enum WikiError {
    #[error("invalid input: {0}")]
    InvalidInput(String),
    #[error("not found: {0}")]
    NotFound(String),
    #[error("bg3.wiki timed out")]
    Timeout,
    #[error("bg3.wiki is temporarily unavailable")]
    Unavailable,
    #[error("bg3.wiki rejected the request: {0}")]
    Rejected(String),
    #[error("bg3.wiki returned an unexpected response")]
    UnexpectedResponse,
}

#[derive(Debug, Error, Clone)]
pub enum ModIoError {
    #[error("invalid input: {0}")]
    InvalidInput(String),
    #[error("mod not found")]
    NotFound,
    #[error("mod.io timed out")]
    Timeout,
    #[error("mod.io is temporarily unavailable")]
    Unavailable,
    #[error("mod.io credentials were rejected")]
    Unauthorized,
    #[error("mod.io rejected the request")]
    Rejected,
    #[error("mod.io returned an unexpected response")]
    UnexpectedResponse,
}

impl WikiError {
    pub fn public_message(&self) -> String {
        self.to_string()
    }
}

impl ModIoError {
    pub fn public_message(&self) -> String {
        self.to_string()
    }
}
