mod common;

use std::{
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    time::Duration,
};

use bg3_mcp::{
    ModIoError,
    mods::{
        ModsService,
        models::{GetModInput, ModPlatform, ModSort, SearchModsInput},
    },
};
use serde_json::json;
use wiremock::{
    Mock, MockServer, Request, Respond, ResponseTemplate,
    matchers::{header, method, path, query_param},
};

use common::test_config;

#[tokio::test]
async fn search_sends_filters_returns_cursor_and_caches_success() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/games/6715/mods"))
        .and(query_param("api_key", "test-api-key"))
        .and(query_param("_q", "dice"))
        .and(query_param("_limit", "5"))
        .and(query_param("_offset", "1"))
        .and(query_param("_sort", "-downloads_total"))
        .and(header("X-Modio-Platform", "ps5"))
        .respond_with(search_response())
        .expect(1)
        .mount(&server)
        .await;
    let service = ModsService::from_config(&test_config(&server.uri())).unwrap();
    let input = SearchModsInput {
        query: Some(" dice ".to_string()),
        platform: ModPlatform::Ps5,
        sort: ModSort::Downloads,
        limit: Some(5),
        cursor: Some(1),
    };

    let response = service.search(input.clone()).await.unwrap();
    service.search(input).await.unwrap();

    assert_eq!(response.query.as_deref(), Some("dice"));
    assert_eq!(response.result_total, 8);
    assert_eq!(response.next_cursor, Some(3));
    assert_eq!(response.results[0].id, 42);
    assert_eq!(
        response.results[0].current_version.as_deref(),
        Some("1.2.0")
    );
    assert_eq!(response.results[0].stats.downloads_total, Some(1234));
    assert_eq!(response.attribution.name, "mod.io");
}

#[tokio::test]
async fn get_returns_plaintext_metadata_without_download_url() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/games/6715/mods/42"))
        .and(query_param("api_key", "test-api-key"))
        .and(header("X-Modio-Platform", "windows"))
        .respond_with(ResponseTemplate::new(200).set_body_json(mod_json()))
        .mount(&server)
        .await;
    let service = ModsService::from_config(&test_config(&server.uri())).unwrap();

    let response = service
        .get(GetModInput {
            mod_id: 42,
            platform: ModPlatform::Windows,
        })
        .await
        .unwrap();
    let serialized = serde_json::to_string(&response).unwrap();

    assert_eq!(response.description, "Roll better dice.");
    assert_eq!(response.current_file.unwrap().id, 99);
    assert!(
        response
            .credit_options
            .contains(&"redistribution_with_credit".to_string())
    );
    assert!(!serialized.contains("binary_url"));
    assert!(!serialized.contains("secret-download"));
}

#[tokio::test]
async fn validates_inputs_before_calling_modio() {
    let service = ModsService::from_config(&test_config("http://127.0.0.1:9")).unwrap();

    let error = service
        .search(SearchModsInput {
            query: Some("   ".to_string()),
            platform: ModPlatform::Windows,
            sort: ModSort::Updated,
            limit: Some(21),
            cursor: None,
        })
        .await
        .unwrap_err();
    assert!(matches!(error, ModIoError::InvalidInput(_)));

    let error = service
        .get(GetModInput {
            mod_id: 0,
            platform: ModPlatform::Windows,
        })
        .await
        .unwrap_err();
    assert!(matches!(error, ModIoError::InvalidInput(_)));
}

#[tokio::test]
async fn maps_not_found_and_malformed_responses() {
    let not_found = MockServer::start().await;
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(404))
        .mount(&not_found)
        .await;
    let service = ModsService::from_config(&test_config(&not_found.uri())).unwrap();
    let error = service
        .get(GetModInput {
            mod_id: 42,
            platform: ModPlatform::Windows,
        })
        .await
        .unwrap_err();
    assert!(matches!(error, ModIoError::NotFound));

    let malformed = MockServer::start().await;
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(200).set_body_string("not-json"))
        .mount(&malformed)
        .await;
    let service = ModsService::from_config(&test_config(&malformed.uri())).unwrap();
    let error = service
        .get(GetModInput {
            mod_id: 42,
            platform: ModPlatform::Windows,
        })
        .await
        .unwrap_err();
    assert!(matches!(error, ModIoError::UnexpectedResponse));
}

#[tokio::test]
async fn maps_timeout_without_exposing_request_details() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(200).set_delay(Duration::from_millis(100)))
        .mount(&server)
        .await;
    let mut config = test_config(&server.uri());
    config.http_timeout = Duration::from_millis(10);
    let service = ModsService::from_config(&config).unwrap();

    let error = service
        .get(GetModInput {
            mod_id: 42,
            platform: ModPlatform::Windows,
        })
        .await
        .unwrap_err();

    assert!(matches!(error, ModIoError::Timeout));
    assert_eq!(error.public_message(), "mod.io timed out");
}

#[tokio::test]
async fn retries_rate_limit_responses() {
    let server = MockServer::start().await;
    let calls = Arc::new(AtomicUsize::new(0));
    Mock::given(method("GET"))
        .respond_with(RetryThenSuccess {
            calls: Arc::clone(&calls),
            first_status: 429,
            retry_after: Some("1"),
        })
        .expect(2)
        .mount(&server)
        .await;
    let mut config = test_config(&server.uri());
    config.http_retry_max = 1;
    let service = ModsService::from_config(&config).unwrap();

    let response = service
        .get(GetModInput {
            mod_id: 42,
            platform: ModPlatform::Windows,
        })
        .await
        .unwrap();

    assert_eq!(response.id, 42);
    assert_eq!(calls.load(Ordering::SeqCst), 2);
}

#[tokio::test]
async fn retries_server_errors() {
    let server = MockServer::start().await;
    let calls = Arc::new(AtomicUsize::new(0));
    Mock::given(method("GET"))
        .respond_with(RetryThenSuccess {
            calls: Arc::clone(&calls),
            first_status: 503,
            retry_after: None,
        })
        .expect(2)
        .mount(&server)
        .await;
    let mut config = test_config(&server.uri());
    config.http_retry_max = 1;
    let service = ModsService::from_config(&config).unwrap();

    let response = service
        .get(GetModInput {
            mod_id: 42,
            platform: ModPlatform::Windows,
        })
        .await
        .unwrap();

    assert_eq!(response.id, 42);
    assert_eq!(calls.load(Ordering::SeqCst), 2);
}

#[tokio::test]
async fn rejects_invalid_credentials_without_exposing_them() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(401).set_body_json(json!({
            "error": {"message": "test-api-key is invalid"}
        })))
        .mount(&server)
        .await;
    let service = ModsService::from_config(&test_config(&server.uri())).unwrap();

    let error = service
        .get(GetModInput {
            mod_id: 42,
            platform: ModPlatform::Windows,
        })
        .await
        .unwrap_err();

    assert!(matches!(error, ModIoError::Unauthorized));
    assert!(!error.public_message().contains("test-api-key"));
}

#[tokio::test]
async fn returns_large_descriptions_without_truncation() {
    let server = MockServer::start().await;
    let content = "x".repeat(1_000_000);
    let mut body = mod_json();
    body["description_plaintext"] = json!(content);
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(200).set_body_json(body))
        .mount(&server)
        .await;
    let service = ModsService::from_config(&test_config(&server.uri())).unwrap();

    let response = service
        .get(GetModInput {
            mod_id: 42,
            platform: ModPlatform::Windows,
        })
        .await
        .unwrap();

    assert_eq!(response.description.len(), 1_000_000);
}

#[tokio::test]
async fn accepts_null_plaintext_descriptions() {
    let server = MockServer::start().await;
    let mut body = mod_json();
    body["description_plaintext"] = serde_json::Value::Null;
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(200).set_body_json(body))
        .mount(&server)
        .await;
    let service = ModsService::from_config(&test_config(&server.uri())).unwrap();

    let response = service
        .get(GetModInput {
            mod_id: 42,
            platform: ModPlatform::Windows,
        })
        .await
        .unwrap();

    assert!(response.description.is_empty());
}

#[tokio::test]
async fn preserves_the_configured_v1_base_path() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/games/6715/mods/42"))
        .respond_with(ResponseTemplate::new(200).set_body_json(mod_json()))
        .mount(&server)
        .await;
    let base_url = format!("{}/v1/", server.uri());
    let service = ModsService::from_config(&test_config(&base_url)).unwrap();

    let response = service
        .get(GetModInput {
            mod_id: 42,
            platform: ModPlatform::Windows,
        })
        .await
        .unwrap();

    assert_eq!(response.id, 42);
}

fn search_response() -> ResponseTemplate {
    ResponseTemplate::new(200).set_body_json(json!({
        "data": [mod_json(), mod_json()],
        "result_count": 2,
        "result_offset": 1,
        "result_limit": 5,
        "result_total": 8
    }))
}

fn mod_json() -> serde_json::Value {
    json!({
        "id": 42,
        "name": "Better Dice",
        "name_id": "better-dice",
        "summary": "Better looking dice.",
        "description": "<p>Roll better dice.</p>",
        "description_plaintext": "Roll better dice.",
        "profile_url": "https://mod.io/g/baldursgate3/m/better-dice",
        "homepage_url": "",
        "submitted_by": {
            "id": 7,
            "username": "Author",
            "profile_url": "https://mod.io/u/author"
        },
        "date_added": 100,
        "date_updated": 200,
        "date_live": 150,
        "maturity_option": 4,
        "credit_options": 5,
        "logo": {
            "original": "https://image.modcdn.io/logo.png",
            "thumb_640x360": "https://thumb.modcdn.io/logo.png"
        },
        "media": {
            "youtube": ["https://youtube.example/video"],
            "images": [{"original": "https://image.modcdn.io/image.png"}]
        },
        "modfile": {
            "id": 99,
            "version": "1.2.0",
            "filename": "better-dice.zip",
            "changelog": "Updated textures",
            "filesize": 1024,
            "filesize_uncompressed": 2048,
            "date_updated": 200,
            "virus_status": 1,
            "virus_positive": 0,
            "download": {"binary_url": "https://secret-download.example/file"},
            "platforms": [{"platform": "windows", "status": 1}]
        },
        "dependencies": true,
        "platforms": [{"platform": "windows", "modfile_live": 99}],
        "tags": [{"name": "Dice", "name_localized": "Dice"}],
        "stats": {
            "downloads_total": 1234,
            "subscribers_total": 500,
            "ratings_total": 25,
            "ratings_percentage_positive": 96,
            "ratings_weighted_aggregate": 91.5,
            "ratings_display_text": "Very Positive"
        }
    })
}

#[derive(Clone)]
struct RetryThenSuccess {
    calls: Arc<AtomicUsize>,
    first_status: u16,
    retry_after: Option<&'static str>,
}

impl Respond for RetryThenSuccess {
    fn respond(&self, _request: &Request) -> ResponseTemplate {
        if self.calls.fetch_add(1, Ordering::SeqCst) == 0 {
            let response = ResponseTemplate::new(self.first_status);
            if let Some(retry_after) = self.retry_after {
                response.insert_header("Retry-After", retry_after)
            } else {
                response
            }
        } else {
            ResponseTemplate::new(200).set_body_json(mod_json())
        }
    }
}
