use agentshield_rs::build_app;
use axum::{
    body::Body,
    http::{Request, StatusCode, header},
};
use serde_json::{Value, json};
use tower::ServiceExt;

async fn post_analyze(payload: Value) -> (StatusCode, Value) {
    let app = build_app();

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/analyze")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    let status = response.status();

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();

    let json: Value = serde_json::from_slice(&body).unwrap();

    (status, json)
}

#[tokio::test]
async fn health_check_returns_ok() {
    let app = build_app();

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();

    assert_eq!(&body[..], b"ok");
}

#[tokio::test]
async fn analyzes_native_transfer() {
    let payload = json!({
        "chain_id": 1,
        "from": "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        "to": "0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
        "value": "1000000000000000000",
        "data": "0x"
    });

    let (status, body) = post_analyze(payload).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["risk_score"], 15);
    assert_eq!(body["risk_level"], "low");
    assert_eq!(body["recommendation"], "allow");
    assert_eq!(body["summary"], "Native token transfer detected");

    assert_eq!(body["decoded_action"]["type"], "native_transfer");
    assert_eq!(
        body["decoded_action"]["recipient"],
        "0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"
    );
    assert_eq!(body["decoded_action"]["amount"], "1000000000000000000");

    assert_eq!(body["findings"][0]["code"], "NATIVE_TRANSFER");
}

#[tokio::test]
async fn analyzes_unlimited_erc20_approval() {
    let payload = json!({
        "chain_id": 1,
        "from": "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        "to": "0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
        "value": "0",
        "data": "0x095ea7b30000000000000000000000001111111111111111111111111111111111111111ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
    });

    let (status, body) = post_analyze(payload).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["risk_score"], 90);
    assert_eq!(body["risk_level"], "critical");
    assert_eq!(body["recommendation"], "reject");
    assert_eq!(body["summary"], "Unlimited ERC-20 approval detected");

    assert_eq!(body["decoded_action"]["type"], "erc20_approve");
    assert_eq!(
        body["decoded_action"]["token"],
        "0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"
    );
    assert_eq!(
        body["decoded_action"]["spender"],
        "0x1111111111111111111111111111111111111111"
    );
    assert_eq!(body["decoded_action"]["amount"], "unlimited");

    let finding_codes: Vec<&str> = body["findings"]
        .as_array()
        .unwrap()
        .iter()
        .map(|finding| finding["code"].as_str().unwrap())
        .collect();

    assert!(finding_codes.contains(&"ERC20_APPROVAL"));
    assert!(finding_codes.contains(&"UNLIMITED_APPROVAL"));
}

#[tokio::test]
async fn analyzes_limited_erc20_approval() {
    let payload = json!({
        "chain_id": 1,
        "from": "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        "to": "0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
        "value": "0",
        "data": "0x095ea7b300000000000000000000000011111111111111111111111111111111111111110000000000000000000000000000000000000000000000000000000000000064"
    });

    let (status, body) = post_analyze(payload).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["risk_score"], 55);
    assert_eq!(body["risk_level"], "high");
    assert_eq!(body["recommendation"], "review");
    assert_eq!(body["summary"], "ERC-20 approval detected");

    assert_eq!(body["decoded_action"]["type"], "erc20_approve");
    assert_eq!(
        body["decoded_action"]["spender"],
        "0x1111111111111111111111111111111111111111"
    );
    assert_eq!(
        body["decoded_action"]["amount"],
        "0x0000000000000000000000000000000000000000000000000000000000000064"
    );
}

#[tokio::test]
async fn analyzes_erc20_transfer() {
    let payload = json!({
        "chain_id": 1,
        "from": "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        "to": "0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
        "value": "0",
        "data": "0xa9059cbb00000000000000000000000022222222222222222222222222222222222222220000000000000000000000000000000000000000000000000000000000000064"
    });

    let (status, body) = post_analyze(payload).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["risk_score"], 35);
    assert_eq!(body["risk_level"], "medium");
    assert_eq!(body["recommendation"], "review");
    assert_eq!(body["summary"], "ERC-20 token transfer detected");

    assert_eq!(body["decoded_action"]["type"], "erc20_transfer");
    assert_eq!(
        body["decoded_action"]["token"],
        "0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"
    );
    assert_eq!(
        body["decoded_action"]["recipient"],
        "0x2222222222222222222222222222222222222222"
    );
    assert_eq!(
        body["decoded_action"]["amount"],
        "0x0000000000000000000000000000000000000000000000000000000000000064"
    );

    assert_eq!(body["findings"][0]["code"], "ERC20_TRANSFER");
}

#[tokio::test]
async fn analyzes_unknown_contract_call() {
    let payload = json!({
        "chain_id": 1,
        "from": "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        "to": "0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
        "value": "0",
        "data": "0xdeadbeef0000000000000000000000001111111111111111111111111111111111111111"
    });

    let (status, body) = post_analyze(payload).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["risk_score"], 70);
    assert_eq!(body["risk_level"], "high");
    assert_eq!(body["recommendation"], "review");
    assert_eq!(body["summary"], "Unknown contract call detected");

    assert_eq!(body["decoded_action"]["type"], "unknown_call");
    assert_eq!(
        body["decoded_action"]["target"],
        "0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"
    );

    assert_eq!(body["findings"][0]["code"], "UNKNOWN_CALLDATA");
}

#[tokio::test]
async fn malformed_erc20_approval_becomes_unknown_call() {
    let payload = json!({
        "chain_id": 1,
        "from": "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        "to": "0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
        "value": "0",
        "data": "0x095ea7b3abcd"
    });

    let (status, body) = post_analyze(payload).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["risk_score"], 70);
    assert_eq!(body["risk_level"], "high");
    assert_eq!(body["recommendation"], "review");
    assert_eq!(body["decoded_action"]["type"], "unknown_call");
    assert_eq!(body["findings"][0]["code"], "UNKNOWN_CALLDATA");
}

#[tokio::test]
async fn invalid_json_returns_bad_request() {
    let app = build_app();

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/analyze")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from("{ invalid json"))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}
