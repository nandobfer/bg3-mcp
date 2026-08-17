use percent_encoding::{NON_ALPHANUMERIC, utf8_percent_encode};
use serde::de::DeserializeOwned;
use url::Url;

use crate::{error::WikiError, infrastructure::MediaWikiHttpClient};

use super::models::{ParseApiResponse, QueryApiResponse, RestPageResponse, SearchApiResponse};

#[derive(Clone)]
pub struct MediaWikiClient {
    http: MediaWikiHttpClient,
    base_url: Url,
}

impl MediaWikiClient {
    pub fn new(http: MediaWikiHttpClient, base_url: Url) -> Self {
        Self { http, base_url }
    }

    pub(crate) async fn search(
        &self,
        query: &str,
        limit: u32,
        cursor: Option<u64>,
    ) -> Result<SearchApiResponse, WikiError> {
        let mut params = vec![
            ("action", "query".to_string()),
            ("list", "search".to_string()),
            ("srsearch", query.to_string()),
            ("srlimit", limit.to_string()),
            ("srprop", "snippet|size|wordcount|timestamp".to_string()),
        ];
        if let Some(cursor) = cursor {
            params.push(("sroffset", cursor.to_string()));
        }
        self.action(params).await
    }

    pub(crate) async fn resolve_page(
        &self,
        title: &str,
        include_extract: bool,
    ) -> Result<QueryApiResponse, WikiError> {
        let mut props = "info|revisions|categories".to_string();
        if include_extract {
            props.push_str("|extracts");
        }
        let mut params = vec![
            ("action", "query".to_string()),
            ("titles", title.to_string()),
            ("redirects", "1".to_string()),
            ("prop", props),
            ("inprop", "url".to_string()),
            ("rvprop", "ids|timestamp".to_string()),
            ("cllimit", "max".to_string()),
        ];
        if include_extract {
            params.push(("explaintext", "1".to_string()));
        }
        self.action(params).await
    }

    pub(crate) async fn parse_sections(&self, title: &str) -> Result<ParseApiResponse, WikiError> {
        self.action(vec![
            ("action", "parse".to_string()),
            ("page", title.to_string()),
            ("prop", "sections".to_string()),
        ])
        .await
    }

    pub(crate) async fn parse_content(
        &self,
        title: &str,
        section_index: Option<&str>,
        wikitext: bool,
    ) -> Result<ParseApiResponse, WikiError> {
        let mut params = vec![
            ("action", "parse".to_string()),
            ("page", title.to_string()),
            (
                "prop",
                if wikitext { "wikitext" } else { "text" }.to_string(),
            ),
        ];
        if let Some(index) = section_index {
            params.push(("section", index.to_string()));
        }
        self.action(params).await
    }

    pub(crate) async fn page_source(&self, title: &str) -> Result<RestPageResponse, WikiError> {
        let encoded = encode_title(title);
        let value = self.http.rest_page(&encoded).await?;
        serde_json::from_value(value).map_err(|_| WikiError::UnexpectedResponse)
    }

    pub(crate) async fn links(
        &self,
        title: &str,
        limit: u32,
        cursor: Option<&str>,
    ) -> Result<QueryApiResponse, WikiError> {
        let mut params = vec![
            ("action", "query".to_string()),
            ("titles", title.to_string()),
            ("redirects", "1".to_string()),
            ("prop", "info|links".to_string()),
            ("inprop", "url".to_string()),
            ("plnamespace", "0".to_string()),
            ("pllimit", limit.to_string()),
        ];
        if let Some(cursor) = cursor {
            params.push(("plcontinue", cursor.to_string()));
            params.push(("continue", "||".to_string()));
        }
        self.action(params).await
    }

    pub(crate) fn page_url(&self, title: &str) -> String {
        let title = title.replace(' ', "_");
        let encoded = encode_title(&title);
        self.base_url
            .join(&format!("/wiki/{encoded}"))
            .map(|url| url.to_string())
            .unwrap_or_else(|_| self.base_url.to_string())
    }

    async fn action<T>(&self, params: Vec<(&str, String)>) -> Result<T, WikiError>
    where
        T: DeserializeOwned,
    {
        let value = self.http.action(params).await?;
        serde_json::from_value(value).map_err(|_| WikiError::UnexpectedResponse)
    }
}

fn encode_title(title: &str) -> String {
    utf8_percent_encode(title, NON_ALPHANUMERIC).to_string()
}
