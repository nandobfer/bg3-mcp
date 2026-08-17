use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, JsonSchema)]
pub struct SearchInput {
    /// Text to search for in bg3.wiki.
    pub query: String,
    /// Number of results to return. Defaults to 5 and cannot exceed 20.
    pub limit: Option<u32>,
    /// Continuation offset returned by a previous call.
    pub cursor: Option<u64>,
}

#[derive(Clone, Debug, Deserialize, JsonSchema)]
pub struct GetPageInput {
    /// Page title to retrieve.
    pub title: String,
    /// Content format: text, html, or wikitext. Defaults to text.
    #[serde(default)]
    pub format: PageFormat,
}

#[derive(Clone, Debug, Deserialize, JsonSchema)]
pub struct GetSectionInput {
    /// Page title containing the section or anchor.
    pub title: String,
    /// Section heading, anchor, or numeric MediaWiki section index.
    pub section: String,
    /// Content format: text, html, or wikitext. Defaults to text.
    #[serde(default)]
    pub format: PageFormat,
}

#[derive(Clone, Debug, Deserialize, JsonSchema)]
pub struct GetLinksInput {
    /// Page title whose internal article links should be listed.
    pub title: String,
    /// Number of links to return. Defaults to 25 and cannot exceed 100.
    pub limit: Option<u32>,
    /// Opaque continuation token returned by a previous call.
    pub cursor: Option<String>,
}

#[derive(Clone, Debug, Deserialize, JsonSchema)]
pub struct GetMetadataInput {
    /// Page title whose metadata should be retrieved.
    pub title: String,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum PageFormat {
    #[default]
    Text,
    Html,
    Wikitext,
}

#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
pub struct Attribution {
    pub name: String,
    pub url: String,
    pub license: String,
    pub license_url: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
pub struct Revision {
    pub id: u64,
    pub timestamp: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
pub struct RedirectInfo {
    pub from: String,
    pub to: String,
    pub fragment: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
pub struct SearchResult {
    pub page_id: u64,
    pub title: String,
    pub url: String,
    pub snippet: String,
    pub word_count: u64,
    pub size: u64,
    pub timestamp: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
pub struct SearchResponse {
    pub query: String,
    pub results: Vec<SearchResult>,
    pub next_cursor: Option<u64>,
    pub attribution: Attribution,
}

#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
pub struct SectionDescriptor {
    pub index: Option<String>,
    pub heading: Option<String>,
    pub anchor: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
pub struct PageContentResponse {
    pub requested_title: String,
    pub canonical_title: String,
    pub canonical_url: String,
    pub page_id: u64,
    pub revision: Option<Revision>,
    pub redirect: Option<RedirectInfo>,
    pub content_format: PageFormat,
    pub content_scope: String,
    pub section: Option<SectionDescriptor>,
    pub content: String,
    pub attribution: Attribution,
}

#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
pub struct WikiLink {
    pub title: String,
    pub url: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
pub struct LinksResponse {
    pub requested_title: String,
    pub canonical_title: String,
    pub canonical_url: String,
    pub page_id: u64,
    pub links: Vec<WikiLink>,
    pub next_cursor: Option<String>,
    pub attribution: Attribution,
}

#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
pub struct MetadataResponse {
    pub requested_title: String,
    pub canonical_title: String,
    pub canonical_url: String,
    pub page_id: u64,
    pub content_model: Option<String>,
    pub revision: Option<Revision>,
    pub categories: Vec<String>,
    pub categories_complete: bool,
    pub redirect: Option<RedirectInfo>,
    pub attribution: Attribution,
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct SearchApiResponse {
    pub query: SearchQuery,
    #[serde(rename = "continue")]
    pub continuation: Option<SearchContinuation>,
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct SearchQuery {
    #[serde(default)]
    pub search: Vec<SearchApiItem>,
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct SearchContinuation {
    pub sroffset: u64,
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct SearchApiItem {
    pub pageid: u64,
    pub title: String,
    #[serde(default)]
    pub size: u64,
    #[serde(default)]
    pub wordcount: u64,
    #[serde(default)]
    pub snippet: String,
    #[serde(default)]
    pub timestamp: String,
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct QueryApiResponse {
    pub query: QueryData,
    #[serde(rename = "continue")]
    pub continuation: Option<QueryContinuation>,
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct QueryData {
    #[serde(default)]
    pub redirects: Vec<RedirectApi>,
    #[serde(default)]
    pub pages: Vec<PageApi>,
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct QueryContinuation {
    pub plcontinue: Option<String>,
    pub clcontinue: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct RedirectApi {
    pub from: String,
    pub to: String,
    pub tofragment: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct PageApi {
    pub pageid: Option<u64>,
    pub title: String,
    pub missing: Option<bool>,
    pub canonicalurl: Option<String>,
    pub fullurl: Option<String>,
    pub contentmodel: Option<String>,
    #[serde(default)]
    pub revisions: Vec<RevisionApi>,
    #[serde(default)]
    pub categories: Vec<CategoryApi>,
    #[serde(default)]
    pub links: Vec<LinkApi>,
    pub extract: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct RevisionApi {
    pub revid: u64,
    #[serde(default)]
    pub timestamp: String,
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct CategoryApi {
    pub title: String,
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct LinkApi {
    pub title: String,
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct ParseApiResponse {
    pub parse: ParseData,
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct ParseData {
    pub text: Option<String>,
    pub wikitext: Option<String>,
    #[serde(default)]
    pub sections: Vec<SectionApi>,
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct SectionApi {
    pub index: String,
    pub line: String,
    pub anchor: String,
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct RestPageResponse {
    pub source: String,
}
