mod common;

use std::{
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    time::Duration,
};

use bg3_mcp::WikiError;
use bg3_mcp::wiki::{
    WikiService,
    models::{GetLinksInput, GetMetadataInput, GetPageInput, PageFormat, SearchInput},
};
use serde_json::json;
use wiremock::{
    Mock, MockServer, Request, Respond, ResponseTemplate,
    matchers::{body_string_contains, method, path},
};

use common::test_config;

#[tokio::test]
async fn search_sanitizes_snippets_and_returns_cursor_and_attribution() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/w/api.php"))
        .and(body_string_contains("list=search"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "continue": { "sroffset": 5, "continue": "-||" },
            "query": {
                "search": [{
                    "pageid": 2403,
                    "title": "Feats",
                    "size": 22482,
                    "wordcount": 129,
                    "snippet": "Great <span class=\"searchmatch\">Weapon</span> Master",
                    "timestamp": "2026-08-06T08:30:22Z"
                }]
            }
        })))
        .mount(&server)
        .await;
    let service = WikiService::from_config(&test_config(&server.uri())).unwrap();

    let response = service
        .search(SearchInput {
            query: "Great Weapon Master".to_string(),
            limit: Some(5),
            cursor: None,
        })
        .await
        .unwrap();

    assert_eq!(response.results[0].snippet, "Great Weapon Master");
    assert_eq!(response.next_cursor, Some(5));
    assert_eq!(response.attribution.name, "bg3.wiki");
    assert!(response.results[0].url.contains("/wiki/Feats"));
}

#[tokio::test]
async fn get_page_returns_complete_untruncated_extract() {
    let server = MockServer::start().await;
    let content = "x".repeat(1_100_000);
    Mock::given(method("POST"))
        .and(path("/w/api.php"))
        .and(body_string_contains("titles=Baldur%27s+Gate+3"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "query": {
                "pages": [{
                    "pageid": 3500,
                    "title": "Baldur's Gate 3",
                    "canonicalurl": "https://bg3.wiki/wiki/Baldur%27s_Gate_3",
                    "contentmodel": "wikitext",
                    "revisions": [{"revid": 397217, "timestamp": "2026-06-27T00:30:50Z"}],
                    "categories": [],
                    "extract": content
                }]
            }
        })))
        .mount(&server)
        .await;
    let service = WikiService::from_config(&test_config(&server.uri())).unwrap();

    let response = service
        .get_page(GetPageInput {
            title: "Baldur's Gate 3".to_string(),
            format: PageFormat::Text,
        })
        .await
        .unwrap();

    assert_eq!(response.content.len(), 1_100_000);
    assert_eq!(response.content_scope, "page");
}

#[tokio::test]
async fn redirect_to_table_anchor_returns_the_complete_row_group() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/w/api.php"))
        .and(body_string_contains("titles=Great+Weapon+Master"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "query": {
                "redirects": [{
                    "from": "Great Weapon Master",
                    "to": "Feats",
                    "tofragment": "Great Weapon Master"
                }],
                "pages": [{
                    "pageid": 2403,
                    "title": "Feats",
                    "canonicalurl": "https://bg3.wiki/wiki/Feats",
                    "revisions": [{"revid": 406234, "timestamp": "2026-08-06T08:30:22Z"}],
                    "categories": [],
                    "extract": "Wrong page introduction"
                }]
            }
        })))
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/w/api.php"))
        .and(body_string_contains("prop=sections"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "parse": {
                "title": "Feats",
                "pageid": 2403,
                "sections": [{
                    "index": "1",
                    "line": "List of all feats",
                    "anchor": "List_of_all_feats"
                }]
            }
        })))
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/w/api.php"))
        .and(body_string_contains("prop=text"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "parse": {
                "title": "Feats",
                "pageid": 2403,
                "text": "<table><tbody><tr><th rowspan=\"2\" scope=\"rowgroup\">Great Weapon Master</th><th><span id=\"Great_Weapon_Master\"></span>Bonus Attack</th></tr><tr><td>Make another melee weapon attack.</td></tr><tr><th scope=\"rowgroup\">Next Feat</th><td>Not included</td></tr></tbody></table>"
            }
        })))
        .mount(&server)
        .await;
    let service = WikiService::from_config(&test_config(&server.uri())).unwrap();

    let response = service
        .get_page(GetPageInput {
            title: "Great Weapon Master".to_string(),
            format: PageFormat::Text,
        })
        .await
        .unwrap();

    assert_eq!(response.content_scope, "fragment");
    assert!(
        response
            .content
            .contains("Make another melee weapon attack")
    );
    assert!(!response.content.contains("Not included"));
    assert_eq!(response.canonical_title, "Feats");
}

#[tokio::test]
async fn links_preserve_mediawiki_continuation() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/w/api.php"))
        .and(body_string_contains("prop=info%7Clinks"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "continue": {"plcontinue": "2403|0|Ability_Modifier", "continue": "||"},
            "query": {
                "pages": [{
                    "pageid": 2403,
                    "title": "Feats",
                    "canonicalurl": "https://bg3.wiki/wiki/Feats",
                    "links": [{"ns": 0, "title": "Abilities"}]
                }]
            }
        })))
        .mount(&server)
        .await;
    let service = WikiService::from_config(&test_config(&server.uri())).unwrap();

    let response = service
        .get_links(GetLinksInput {
            title: "Feats".to_string(),
            limit: Some(25),
            cursor: None,
        })
        .await
        .unwrap();

    assert_eq!(response.links[0].title, "Abilities");
    assert_eq!(
        response.next_cursor.as_deref(),
        Some("2403|0|Ability_Modifier")
    );
}

#[tokio::test]
async fn metadata_includes_redirect_revision_categories_and_license() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/w/api.php"))
        .and(body_string_contains("titles=Great+Weapon+Master"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "query": {
                "redirects": [{
                    "from": "Great Weapon Master",
                    "to": "Feats",
                    "tofragment": "Great Weapon Master"
                }],
                "pages": [{
                    "pageid": 2403,
                    "title": "Feats",
                    "canonicalurl": "https://bg3.wiki/wiki/Feats",
                    "contentmodel": "wikitext",
                    "revisions": [{"revid": 406234, "timestamp": "2026-08-06T08:30:22Z"}],
                    "categories": [{"ns": 14, "title": "Category:Character creation"}]
                }]
            }
        })))
        .mount(&server)
        .await;
    let service = WikiService::from_config(&test_config(&server.uri())).unwrap();

    let response = service
        .get_metadata(GetMetadataInput {
            title: "Great Weapon Master".to_string(),
        })
        .await
        .unwrap();

    assert_eq!(response.revision.unwrap().id, 406234);
    assert_eq!(response.categories, vec!["Category:Character creation"]);
    assert_eq!(response.redirect.unwrap().to, "Feats");
    assert!(response.attribution.license_url.contains("Copyrights"));
}

#[tokio::test]
async fn successful_action_responses_are_cached() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/w/api.php"))
        .and(body_string_contains("list=search"))
        .respond_with(search_response())
        .expect(1)
        .mount(&server)
        .await;
    let service = WikiService::from_config(&test_config(&server.uri())).unwrap();
    let input = SearchInput {
        query: "Karlach".to_string(),
        limit: Some(5),
        cursor: None,
    };

    service.search(input.clone()).await.unwrap();
    service.search(input).await.unwrap();
}

#[tokio::test]
async fn retries_rate_limit_responses() {
    let server = MockServer::start().await;
    let calls = Arc::new(AtomicUsize::new(0));
    Mock::given(method("POST"))
        .and(path("/w/api.php"))
        .and(body_string_contains("list=search"))
        .respond_with(RetryThenSuccess {
            calls: Arc::clone(&calls),
        })
        .expect(2)
        .mount(&server)
        .await;
    let mut config = test_config(&server.uri());
    config.http_retry_max = 1;
    let service = WikiService::from_config(&config).unwrap();

    let response = service
        .search(SearchInput {
            query: "Karlach".to_string(),
            limit: Some(5),
            cursor: None,
        })
        .await
        .unwrap();

    assert_eq!(response.results[0].title, "Karlach");
    assert_eq!(calls.load(Ordering::SeqCst), 2);
}

#[tokio::test]
async fn normalizes_timeout_without_exposing_response_details() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/w/api.php"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_delay(Duration::from_millis(100))
                .set_body_json(json!({"query": {"search": []}})),
        )
        .mount(&server)
        .await;
    let mut config = test_config(&server.uri());
    config.http_timeout = Duration::from_millis(10);
    let service = WikiService::from_config(&config).unwrap();

    let error = service
        .search(SearchInput {
            query: "Karlach".to_string(),
            limit: None,
            cursor: None,
        })
        .await
        .unwrap_err();

    assert!(matches!(error, WikiError::Timeout));
    assert_eq!(error.public_message(), "bg3.wiki timed out");
}

#[tokio::test]
async fn rejects_malformed_mediawiki_json() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/w/api.php"))
        .respond_with(ResponseTemplate::new(200).set_body_string("not-json"))
        .mount(&server)
        .await;
    let service = WikiService::from_config(&test_config(&server.uri())).unwrap();

    let error = service
        .search(SearchInput {
            query: "Karlach".to_string(),
            limit: None,
            cursor: None,
        })
        .await
        .unwrap_err();

    assert!(matches!(error, WikiError::UnexpectedResponse));
}

fn search_response() -> ResponseTemplate {
    ResponseTemplate::new(200).set_body_json(json!({
        "query": {
            "search": [{
                "pageid": 1155,
                "title": "Karlach",
                "size": 100,
                "wordcount": 20,
                "snippet": "Companion",
                "timestamp": "2026-08-01T00:00:00Z"
            }]
        }
    }))
}

#[derive(Clone)]
struct RetryThenSuccess {
    calls: Arc<AtomicUsize>,
}

impl Respond for RetryThenSuccess {
    fn respond(&self, _request: &Request) -> ResponseTemplate {
        if self.calls.fetch_add(1, Ordering::SeqCst) == 0 {
            ResponseTemplate::new(429).insert_header("Retry-After", "0")
        } else {
            search_response()
        }
    }
}
