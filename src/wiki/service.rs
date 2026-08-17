use percent_encoding::percent_decode_str;
use scraper::{ElementRef, Html, Selector};

use crate::{
    config::Config,
    error::{AppError, WikiError},
    infrastructure::MediaWikiHttpClient,
};

use super::{
    MediaWikiClient,
    models::{
        Attribution, GetLinksInput, GetMetadataInput, GetPageInput, GetSectionInput, LinksResponse,
        MetadataResponse, PageApi, PageContentResponse, PageFormat, RedirectApi, RedirectInfo,
        Revision, SearchInput, SearchResponse, SearchResult, SectionApi, SectionDescriptor,
        WikiLink,
    },
};

const SOURCE_NAME: &str = "bg3.wiki";
const SOURCE_URL: &str = "https://bg3.wiki/";
const LICENSE: &str = "CC BY-NC-SA 4.0 or CC BY-SA 4.0 (content-dependent)";
const LICENSE_URL: &str = "https://bg3.wiki/wiki/bg3wiki:Copyrights";

#[derive(Clone)]
pub struct WikiService {
    client: MediaWikiClient,
}

impl WikiService {
    pub fn from_config(config: &Config) -> Result<Self, AppError> {
        let http = MediaWikiHttpClient::new(config)?;
        let client = MediaWikiClient::new(http, config.wiki_base_url.clone());
        Ok(Self::new(client))
    }

    pub(crate) fn new(client: MediaWikiClient) -> Self {
        Self { client }
    }

    pub async fn search(&self, input: SearchInput) -> Result<SearchResponse, WikiError> {
        validate_text("query", &input.query, 200)?;
        let limit = input.limit.unwrap_or(5);
        if !(1..=20).contains(&limit) {
            return Err(WikiError::InvalidInput(
                "limit must be between 1 and 20".to_string(),
            ));
        }

        let response = self
            .client
            .search(&input.query, limit, input.cursor)
            .await?;
        let results = response
            .query
            .search
            .into_iter()
            .map(|item| SearchResult {
                page_id: item.pageid,
                url: self.client.page_url(&item.title),
                title: item.title,
                snippet: html_to_text(&item.snippet),
                word_count: item.wordcount,
                size: item.size,
                timestamp: item.timestamp,
            })
            .collect();

        Ok(SearchResponse {
            query: input.query,
            results,
            next_cursor: response.continuation.map(|value| value.sroffset),
            attribution: attribution(),
        })
    }

    pub async fn get_page(&self, input: GetPageInput) -> Result<PageContentResponse, WikiError> {
        validate_text("title", &input.title, 255)?;
        let resolved = self
            .resolve(&input.title, matches!(input.format, PageFormat::Text))
            .await?;

        if let Some(fragment) = resolved
            .redirect
            .as_ref()
            .and_then(|redirect| redirect.fragment.clone())
        {
            return self
                .content_for_fragment(resolved, &fragment, input.format)
                .await;
        }

        let content = match input.format {
            PageFormat::Text => resolved.page.extract.clone().unwrap_or_default(),
            PageFormat::Html => {
                let parsed = self
                    .client
                    .parse_content(&resolved.page.title, None, false)
                    .await?;
                sanitize_html(parsed.parse.text.as_deref().unwrap_or_default())
            }
            PageFormat::Wikitext => self.client.page_source(&resolved.page.title).await?.source,
        };

        Ok(resolved.into_content(input.format, "page", None, content))
    }

    pub async fn get_section(
        &self,
        input: GetSectionInput,
    ) -> Result<PageContentResponse, WikiError> {
        validate_text("title", &input.title, 255)?;
        validate_text("section", &input.section, 200)?;
        let resolved = self.resolve(&input.title, false).await?;
        self.content_for_fragment(resolved, &input.section, input.format)
            .await
    }

    pub async fn get_links(&self, input: GetLinksInput) -> Result<LinksResponse, WikiError> {
        validate_text("title", &input.title, 255)?;
        let limit = input.limit.unwrap_or(25);
        if !(1..=100).contains(&limit) {
            return Err(WikiError::InvalidInput(
                "limit must be between 1 and 100".to_string(),
            ));
        }
        if input.cursor.as_ref().is_some_and(|value| value.len() > 512) {
            return Err(WikiError::InvalidInput(
                "cursor cannot exceed 512 bytes".to_string(),
            ));
        }

        let response = self
            .client
            .links(&input.title, limit, input.cursor.as_deref())
            .await?;
        let page = required_page(response.query.pages.into_iter().next(), &input.title)?;
        let page_id = page.pageid.ok_or(WikiError::UnexpectedResponse)?;
        let canonical_url = canonical_url(&self.client, &page);
        let links = page
            .links
            .into_iter()
            .map(|link| WikiLink {
                url: self.client.page_url(&link.title),
                title: link.title,
            })
            .collect();

        Ok(LinksResponse {
            requested_title: input.title,
            canonical_title: page.title,
            canonical_url,
            page_id,
            links,
            next_cursor: response.continuation.and_then(|value| value.plcontinue),
            attribution: attribution(),
        })
    }

    pub async fn get_metadata(
        &self,
        input: GetMetadataInput,
    ) -> Result<MetadataResponse, WikiError> {
        validate_text("title", &input.title, 255)?;
        let response = self.client.resolve_page(&input.title, false).await?;
        let categories_complete = response
            .continuation
            .as_ref()
            .and_then(|value| value.clcontinue.as_ref())
            .is_none();
        let redirect_api = response.query.redirects.first().cloned();
        let page = required_page(response.query.pages.into_iter().next(), &input.title)?;
        let page_id = page.pageid.ok_or(WikiError::UnexpectedResponse)?;
        let canonical_url = canonical_url(&self.client, &page);
        let revision = page.revisions.first().map(|revision| Revision {
            id: revision.revid,
            timestamp: revision.timestamp.clone(),
        });
        let redirect = redirect_api.map(redirect_from_api);

        Ok(MetadataResponse {
            requested_title: input.title,
            canonical_title: page.title,
            canonical_url,
            page_id,
            content_model: page.contentmodel,
            revision,
            categories: page.categories.into_iter().map(|item| item.title).collect(),
            categories_complete,
            redirect,
            attribution: attribution(),
        })
    }

    async fn resolve(&self, title: &str, include_extract: bool) -> Result<ResolvedPage, WikiError> {
        let response = self.client.resolve_page(title, include_extract).await?;
        let redirect = response
            .query
            .redirects
            .first()
            .cloned()
            .map(redirect_from_api);
        let page = required_page(response.query.pages.into_iter().next(), title)?;
        let page_id = page.pageid.ok_or(WikiError::UnexpectedResponse)?;
        let canonical_url = canonical_url(&self.client, &page);
        let revision = page.revisions.first().map(|revision| Revision {
            id: revision.revid,
            timestamp: revision.timestamp.clone(),
        });

        Ok(ResolvedPage {
            requested_title: title.to_string(),
            page,
            page_id,
            canonical_url,
            revision,
            redirect,
        })
    }

    async fn content_for_fragment(
        &self,
        resolved: ResolvedPage,
        fragment: &str,
        format: PageFormat,
    ) -> Result<PageContentResponse, WikiError> {
        let sections = self.client.parse_sections(&resolved.page.title).await?;
        if let Some(section) = find_section(&sections.parse.sections, fragment) {
            let parsed = self
                .client
                .parse_content(
                    &resolved.page.title,
                    Some(&section.index),
                    matches!(format, PageFormat::Wikitext),
                )
                .await?;
            let content = match format {
                PageFormat::Text => html_to_text(parsed.parse.text.as_deref().unwrap_or_default()),
                PageFormat::Html => sanitize_html(parsed.parse.text.as_deref().unwrap_or_default()),
                PageFormat::Wikitext => parsed.parse.wikitext.unwrap_or_default(),
            };
            let descriptor = SectionDescriptor {
                index: Some(section.index.clone()),
                heading: Some(section.line.clone()),
                anchor: section.anchor.clone(),
            };
            return Ok(resolved.into_content(format, "section", Some(descriptor), content));
        }

        if matches!(format, PageFormat::Wikitext) {
            return Err(WikiError::NotFound(format!(
                "fragment '{fragment}' is not a MediaWiki section and cannot be isolated as wikitext"
            )));
        }

        let parsed = self
            .client
            .parse_content(&resolved.page.title, None, false)
            .await?;
        let html = parsed.parse.text.unwrap_or_default();
        let fragment_html = extract_fragment_html(&html, fragment).ok_or_else(|| {
            WikiError::NotFound(format!(
                "section or anchor '{fragment}' in page '{}'",
                resolved.page.title
            ))
        })?;
        let content = match format {
            PageFormat::Text => html_to_text(&fragment_html),
            PageFormat::Html => sanitize_html(&fragment_html),
            PageFormat::Wikitext => unreachable!("wikitext is rejected above"),
        };
        let descriptor = SectionDescriptor {
            index: None,
            heading: None,
            anchor: fragment.to_string(),
        };
        Ok(resolved.into_content(format, "fragment", Some(descriptor), content))
    }
}

struct ResolvedPage {
    requested_title: String,
    page: PageApi,
    page_id: u64,
    canonical_url: String,
    revision: Option<Revision>,
    redirect: Option<RedirectInfo>,
}

impl ResolvedPage {
    fn into_content(
        self,
        format: PageFormat,
        scope: &str,
        section: Option<SectionDescriptor>,
        content: String,
    ) -> PageContentResponse {
        PageContentResponse {
            requested_title: self.requested_title,
            canonical_title: self.page.title,
            canonical_url: self.canonical_url,
            page_id: self.page_id,
            revision: self.revision,
            redirect: self.redirect,
            content_format: format,
            content_scope: scope.to_string(),
            section,
            content,
            attribution: attribution(),
        }
    }
}

fn required_page(page: Option<PageApi>, requested_title: &str) -> Result<PageApi, WikiError> {
    let page = page.ok_or(WikiError::UnexpectedResponse)?;
    if page.missing.unwrap_or(false) || page.pageid.is_none() {
        return Err(WikiError::NotFound(format!("page '{requested_title}'")));
    }
    Ok(page)
}

fn canonical_url(client: &MediaWikiClient, page: &PageApi) -> String {
    page.canonicalurl
        .clone()
        .or_else(|| page.fullurl.clone())
        .unwrap_or_else(|| client.page_url(&page.title))
}

fn redirect_from_api(redirect: RedirectApi) -> RedirectInfo {
    RedirectInfo {
        from: redirect.from,
        to: redirect.to,
        fragment: redirect.tofragment,
    }
}

fn attribution() -> Attribution {
    Attribution {
        name: SOURCE_NAME.to_string(),
        url: SOURCE_URL.to_string(),
        license: LICENSE.to_string(),
        license_url: LICENSE_URL.to_string(),
    }
}

fn validate_text(name: &str, value: &str, max_chars: usize) -> Result<(), WikiError> {
    let length = value.chars().count();
    if value.trim().is_empty() {
        return Err(WikiError::InvalidInput(format!("{name} cannot be empty")));
    }
    if length > max_chars {
        return Err(WikiError::InvalidInput(format!(
            "{name} cannot exceed {max_chars} characters"
        )));
    }
    Ok(())
}

fn normalize_fragment(value: &str) -> String {
    let decoded = percent_decode_str(value).decode_utf8_lossy();
    decoded
        .replace('_', " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

fn find_section<'a>(sections: &'a [SectionApi], fragment: &str) -> Option<&'a SectionApi> {
    let normalized = normalize_fragment(fragment);
    sections.iter().find(|section| {
        section.index == fragment
            || normalize_fragment(&section.line) == normalized
            || normalize_fragment(&section.anchor) == normalized
    })
}

fn sanitize_html(html: &str) -> String {
    ammonia::Builder::default().clean(html).to_string()
}

fn html_to_text(html: &str) -> String {
    let document = Html::parse_fragment(html);
    document
        .root_element()
        .text()
        .flat_map(str::split_whitespace)
        .collect::<Vec<_>>()
        .join(" ")
}

fn extract_fragment_html(html: &str, fragment: &str) -> Option<String> {
    let document = Html::parse_fragment(html);
    let selector = Selector::parse("[id]").ok()?;
    let normalized = normalize_fragment(fragment);
    let target = document.select(&selector).find(|element| {
        element
            .value()
            .attr("id")
            .is_some_and(|id| normalize_fragment(id) == normalized)
    })?;

    if let Some(row) = ancestor_with_tag(target, "tr") {
        return extract_table_row_group(row);
    }

    for tag in ["li", "p", "section", "div", "table"] {
        if let Some(element) = ancestor_with_tag(target, tag) {
            return Some(element.html());
        }
    }

    Some(target.html())
}

fn ancestor_with_tag<'a>(element: ElementRef<'a>, tag: &str) -> Option<ElementRef<'a>> {
    element
        .ancestors()
        .filter_map(ElementRef::wrap)
        .find(|ancestor| ancestor.value().name() == tag)
}

fn extract_table_row_group(row: ElementRef<'_>) -> Option<String> {
    let parent = row.parent().and_then(ElementRef::wrap)?;
    let rows = parent
        .children()
        .filter_map(ElementRef::wrap)
        .filter(|element| element.value().name() == "tr")
        .collect::<Vec<_>>();
    let position = rows
        .iter()
        .position(|candidate| candidate.id() == row.id())?;
    let rowgroup_selector = Selector::parse("th[scope=\"rowgroup\"]").ok()?;
    let count = row
        .select(&rowgroup_selector)
        .next()
        .and_then(|header| header.value().attr("rowspan"))
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(1);
    Some(
        rows.into_iter()
            .skip(position)
            .take(count)
            .map(|element| element.html())
            .collect::<Vec<_>>()
            .join("\n"),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_search_markup() {
        assert_eq!(
            html_to_text("Great <span class=\"searchmatch\">Weapon</span> Master"),
            "Great Weapon Master"
        );
    }

    #[test]
    fn matches_section_heading_anchor_and_index() {
        let sections = vec![SectionApi {
            index: "2".to_string(),
            line: "Notes and references".to_string(),
            anchor: "Notes_and_references".to_string(),
        }];

        assert!(find_section(&sections, "2").is_some());
        assert!(find_section(&sections, "notes and references").is_some());
        assert!(find_section(&sections, "Notes_and_references").is_some());
    }

    #[test]
    fn extracts_all_rows_in_table_rowgroup() {
        let html = r#"
            <table><tbody>
              <tr><th rowspan="2" scope="rowgroup">Feat</th><th><span id="Feat"></span>Name</th></tr>
              <tr><td>Description</td></tr>
              <tr><th rowspan="1" scope="rowgroup">Next</th><td>Other</td></tr>
            </tbody></table>
        "#;

        let extracted = extract_fragment_html(html, "Feat").unwrap();
        assert!(extracted.contains("Description"));
        assert!(!extracted.contains("Other"));
    }
}
