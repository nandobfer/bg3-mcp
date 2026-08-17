mod common;

use axum::{
    body::{Body, to_bytes},
    http::{Method, Request, StatusCode, header},
};
use bg3_mcp::build_router;
use serde_json::{Value, json};
use tokio_util::sync::CancellationToken;
use tower::ServiceExt;
use wiremock::{
    Mock, MockServer, ResponseTemplate,
    matchers::{body_string_contains, method, path},
};

use common::test_config;

#[tokio::test]
async fn health_check_does_not_call_the_wiki() {
    let router =
        build_router(&test_config("http://127.0.0.1:9"), CancellationToken::new()).unwrap();
    let response = router
        .oneshot(Request::get("/health").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let payload: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(payload["status"], "ok");
}

#[tokio::test]
async fn cors_accepts_arbitrary_origins() {
    let router =
        build_router(&test_config("http://127.0.0.1:9"), CancellationToken::new()).unwrap();
    let response = router
        .oneshot(
            Request::builder()
                .method(Method::OPTIONS)
                .uri("/mcp")
                .header(header::ORIGIN, "https://any-community-client.example")
                .header(header::ACCESS_CONTROL_REQUEST_METHOD, "POST")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response
            .headers()
            .get(header::ACCESS_CONTROL_ALLOW_ORIGIN)
            .unwrap(),
        "*"
    );
}

#[tokio::test]
async fn mcp_route_has_no_application_body_limit() {
    let router =
        build_router(&test_config("http://127.0.0.1:9"), CancellationToken::new()).unwrap();
    let payload = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "ping",
        "padding": "x".repeat(3_000_000)
    });
    let response = router
        .oneshot(
            Request::post("/mcp")
                .header(header::HOST, "public-community.example")
                .header(header::ORIGIN, "https://any-community-client.example")
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::ACCEPT, "application/json, text/event-stream")
                .body(Body::from(payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_ne!(response.status(), StatusCode::PAYLOAD_TOO_LARGE);
}

#[tokio::test]
async fn mcp_lists_all_tools() {
    let router =
        build_router(&test_config("http://127.0.0.1:9"), CancellationToken::new()).unwrap();
    let initialize = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "protocolVersion": "2025-11-25",
            "capabilities": {},
            "clientInfo": {"name": "integration-test", "version": "1.0.0"}
        }
    });
    let initialize_response = router
        .clone()
        .oneshot(
            Request::post("/mcp")
                .header(header::HOST, "public-community.example")
                .header(header::ORIGIN, "https://any-community-client.example")
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::ACCEPT, "application/json, text/event-stream")
                .body(Body::from(initialize.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    let initialize_status = initialize_response.status();
    let initialize_body = to_bytes(initialize_response.into_body(), usize::MAX)
        .await
        .unwrap();
    assert_eq!(
        initialize_status,
        StatusCode::OK,
        "unexpected initialize response: {}",
        String::from_utf8_lossy(&initialize_body)
    );

    let request = json!({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "tools/list",
        "params": {
            "_meta": {
                "protocolVersion": "2025-11-25",
                "clientInfo": {"name": "integration-test", "version": "1.0.0"},
                "capabilities": {}
            }
        }
    });
    let response = router
        .oneshot(
            Request::post("/mcp")
                .header(header::HOST, "public-community.example")
                .header(header::ORIGIN, "https://any-community-client.example")
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::ACCEPT, "application/json, text/event-stream")
                .header("mcp-protocol-version", "2025-11-25")
                .body(Body::from(request.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    let status = response.status();
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    assert_eq!(
        status,
        StatusCode::OK,
        "unexpected response: {}",
        String::from_utf8_lossy(&body)
    );
    assert!(
        !String::from_utf8_lossy(&body).contains("\"format\":\"uint"),
        "tool schemas must not expose Rust-specific integer formats"
    );
    let payload: Value = serde_json::from_slice(&body).unwrap();
    let tools = payload["result"]["tools"].as_array().unwrap();
    let names = tools
        .iter()
        .filter_map(|tool| tool["name"].as_str())
        .collect::<Vec<_>>();
    assert_eq!(names.len(), 7);
    assert!(names.contains(&"wiki_search"));
    assert!(names.contains(&"wiki_get_page"));
    assert!(names.contains(&"wiki_get_section"));
    assert!(names.contains(&"wiki_get_links"));
    assert!(names.contains(&"wiki_get_metadata"));
    assert!(names.contains(&"mods_search"));
    assert!(names.contains(&"mods_get"));
}

#[tokio::test]
async fn mcp_search_tool_returns_structured_attributed_content() {
    let wiki = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/w/api.php"))
        .and(body_string_contains("list=search"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "query": {
                "search": [{
                    "pageid": 1155,
                    "title": "Karlach",
                    "size": 100,
                    "wordcount": 20,
                    "snippet": "A <span class=\"searchmatch\">companion</span>",
                    "timestamp": "2026-08-01T00:00:00Z"
                }]
            }
        })))
        .mount(&wiki)
        .await;
    let router = build_router(&test_config(&wiki.uri()), CancellationToken::new()).unwrap();

    let initialize = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "protocolVersion": "2025-11-25",
            "capabilities": {},
            "clientInfo": {"name": "integration-test", "version": "1.0.0"}
        }
    });
    let initialize_response = router
        .clone()
        .oneshot(mcp_request(initialize, false))
        .await
        .unwrap();
    assert_eq!(initialize_response.status(), StatusCode::OK);

    let call = json!({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "tools/call",
        "params": {
            "name": "wiki_search",
            "arguments": {"query": "Karlach", "limit": 1},
            "_meta": {
                "protocolVersion": "2025-11-25",
                "clientInfo": {"name": "integration-test", "version": "1.0.0"},
                "capabilities": {}
            }
        }
    });
    let response = router.oneshot(mcp_request(call, true)).await.unwrap();
    let status = response.status();
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    assert_eq!(
        status,
        StatusCode::OK,
        "unexpected response: {}",
        String::from_utf8_lossy(&body)
    );
    let payload: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(
        payload["result"]["structuredContent"]["results"][0]["title"],
        "Karlach"
    );
    assert_eq!(
        payload["result"]["structuredContent"]["attribution"]["name"],
        "bg3.wiki"
    );
}

#[tokio::test]
async fn mcp_mods_search_returns_structured_attributed_content() {
    let modio = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/games/6715/mods"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "data": [{
                "id": 42,
                "name": "Better Dice",
                "name_id": "better-dice",
                "summary": "Better looking dice.",
                "description_plaintext": "Better looking dice.",
                "profile_url": "https://mod.io/g/baldursgate3/m/better-dice",
                "platforms": [{"platform": "windows"}],
                "tags": [],
                "stats": {}
            }],
            "result_count": 1,
            "result_offset": 0,
            "result_limit": 1,
            "result_total": 1
        })))
        .mount(&modio)
        .await;
    let router = build_router(&test_config(&modio.uri()), CancellationToken::new()).unwrap();
    let initialize = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "protocolVersion": "2025-11-25",
            "capabilities": {},
            "clientInfo": {"name": "integration-test", "version": "1.0.0"}
        }
    });
    let initialize_response = router
        .clone()
        .oneshot(mcp_request(initialize, false))
        .await
        .unwrap();
    assert_eq!(initialize_response.status(), StatusCode::OK);

    let call = json!({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "tools/call",
        "params": {
            "name": "mods_search",
            "arguments": {"query": "dice", "limit": 1},
            "_meta": {
                "protocolVersion": "2025-11-25",
                "clientInfo": {"name": "integration-test", "version": "1.0.0"},
                "capabilities": {}
            }
        }
    });
    let response = router.oneshot(mcp_request(call, true)).await.unwrap();
    let status = response.status();
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    assert_eq!(
        status,
        StatusCode::OK,
        "unexpected response: {}",
        String::from_utf8_lossy(&body)
    );
    let payload: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(
        payload["result"]["structuredContent"]["results"][0]["name"],
        "Better Dice"
    );
    assert_eq!(
        payload["result"]["structuredContent"]["attribution"]["name"],
        "mod.io"
    );
}

fn mcp_request(payload: Value, protocol_header: bool) -> Request<Body> {
    let mut request = Request::post("/mcp")
        .header(header::HOST, "public-community.example")
        .header(header::ORIGIN, "https://any-community-client.example")
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::ACCEPT, "application/json, text/event-stream");
    if protocol_header {
        request = request.header("mcp-protocol-version", "2025-11-25");
    }
    request.body(Body::from(payload.to_string())).unwrap()
}
