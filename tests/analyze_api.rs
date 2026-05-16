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

fn finding_codes(body: &Value) -> Vec<&str> {
    body["findings"]
        .as_array()
        .unwrap()
        .iter()
        .map(|finding| finding["code"].as_str().unwrap())
        .collect()
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

#[tokio::test]
async fn analyzes_native_transfer_to_unknown_recipient() {
    let payload = json!({
        "chain_id": 1,
        "from": "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        "to": "0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
        "value": "1000000000000000000",
        "data": "0x"
    });

    let (status, body) = post_analyze(payload).await;

    assert_eq!(status, StatusCode::OK);

    // Native transfer +10, unknown recipient +20 = 30
    assert_eq!(body["risk_score"], 30);
    assert_eq!(body["risk_level"], "medium");
    assert_eq!(body["recommendation"], "review");
    assert_eq!(body["summary"], "Native token transfer detected");

    assert_eq!(body["decoded_action"]["type"], "native_transfer");
    assert_eq!(
        body["decoded_action"]["recipient"],
        "0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"
    );
    assert_eq!(body["decoded_action"]["amount"], "1000000000000000000");

    let codes = finding_codes(&body);
    assert!(codes.contains(&"NATIVE_TRANSFER"));
    assert!(codes.contains(&"UNKNOWN_RECIPIENT"));
}

#[tokio::test]
async fn analyzes_native_transfer_to_trusted_recipient() {
    let payload = json!({
        "chain_id": 1,
        "from": "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        "to": "0x9999999999999999999999999999999999999999",
        "value": "1000000000000000000",
        "data": "0x"
    });

    let (status, body) = post_analyze(payload).await;

    assert_eq!(status, StatusCode::OK);

    // Native transfer +10, trusted recipient -10 = 0
    assert_eq!(body["risk_score"], 0);
    assert_eq!(body["risk_level"], "low");
    assert_eq!(body["recommendation"], "allow");
    assert_eq!(body["summary"], "Native token transfer detected");

    assert_eq!(body["decoded_action"]["type"], "native_transfer");
    assert_eq!(
        body["decoded_action"]["recipient"],
        "0x9999999999999999999999999999999999999999"
    );

    let codes = finding_codes(&body);
    assert!(codes.contains(&"NATIVE_TRANSFER"));
    assert!(codes.contains(&"TRUSTED_RECIPIENT"));
}

#[tokio::test]
async fn analyzes_unlimited_erc20_approval_to_unknown_spender() {
    let payload = json!({
        "chain_id": 1,
        "from": "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        "to": "0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48",
        "value": "0",
        "data": "0x095ea7b30000000000000000000000001111111111111111111111111111111111111111ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
    });

    let (status, body) = post_analyze(payload).await;

    assert_eq!(status, StatusCode::OK);

    // ERC-20 approval +20, unlimited +50, unknown spender +25 = 95
    assert_eq!(body["risk_score"], 95);
    assert_eq!(body["risk_level"], "critical");
    assert_eq!(body["recommendation"], "reject");
    assert_eq!(
        body["summary"],
        "Unlimited ERC-20 approval to unknown spender"
    );

    assert_eq!(body["decoded_action"]["type"], "erc20_approve");
    assert_eq!(
        body["decoded_action"]["token"],
        "0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48"
    );
    assert_eq!(
        body["decoded_action"]["spender"],
        "0x1111111111111111111111111111111111111111"
    );
    assert_eq!(body["decoded_action"]["amount"], "unlimited");

    let codes = finding_codes(&body);
    assert!(codes.contains(&"ERC20_APPROVAL"));
    assert!(codes.contains(&"KNOWN_TOKEN"));
    assert!(codes.contains(&"UNLIMITED_APPROVAL"));
    assert!(codes.contains(&"UNKNOWN_SPENDER"));
}

#[tokio::test]
async fn analyzes_unlimited_erc20_approval_to_trusted_spender() {
    let payload = json!({
        "chain_id": 1,
        "from": "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        "to": "0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48",
        "value": "0",
        "data": "0x095ea7b30000000000000000000000007a250d5630b4cf539739df2c5dacb4c659f2488dffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
    });

    let (status, body) = post_analyze(payload).await;

    assert_eq!(status, StatusCode::OK);

    // ERC-20 approval +20, unlimited +50, trusted spender -15 = 55
    assert_eq!(body["risk_score"], 55);
    assert_eq!(body["risk_level"], "high");
    assert_eq!(body["recommendation"], "review");
    assert_eq!(body["summary"], "Unlimited ERC-20 approval detected");

    assert_eq!(body["decoded_action"]["type"], "erc20_approve");
    assert_eq!(
        body["decoded_action"]["token"],
        "0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48"
    );
    assert_eq!(
        body["decoded_action"]["spender"],
        "0x7a250d5630b4cf539739df2c5dacb4c659f2488d"
    );
    assert_eq!(body["decoded_action"]["amount"], "unlimited");

    let codes = finding_codes(&body);
    assert!(codes.contains(&"ERC20_APPROVAL"));
    assert!(codes.contains(&"KNOWN_TOKEN"));
    assert!(codes.contains(&"UNLIMITED_APPROVAL"));
    assert!(codes.contains(&"TRUSTED_SPENDER"));
}

#[tokio::test]
async fn analyzes_limited_erc20_approval_to_unknown_spender() {
    let payload = json!({
        "chain_id": 1,
        "from": "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        "to": "0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48",
        "value": "0",
        "data": "0x095ea7b300000000000000000000000011111111111111111111111111111111111111110000000000000000000000000000000000000000000000000000000000000064"
    });

    let (status, body) = post_analyze(payload).await;

    assert_eq!(status, StatusCode::OK);

    // ERC-20 approval +20, unknown spender +25 = 45
    assert_eq!(body["risk_score"], 45);
    assert_eq!(body["risk_level"], "medium");
    assert_eq!(body["recommendation"], "review");
    assert_eq!(body["summary"], "ERC-20 approval to unknown spender");

    assert_eq!(body["decoded_action"]["type"], "erc20_approve");
    assert_eq!(
        body["decoded_action"]["spender"],
        "0x1111111111111111111111111111111111111111"
    );
    assert_eq!(
        body["decoded_action"]["amount"],
        "0x0000000000000000000000000000000000000000000000000000000000000064"
    );

    let codes = finding_codes(&body);
    assert!(codes.contains(&"ERC20_APPROVAL"));
    assert!(codes.contains(&"KNOWN_TOKEN"));
    assert!(codes.contains(&"UNKNOWN_SPENDER"));
    assert!(!codes.contains(&"UNLIMITED_APPROVAL"));
}

#[tokio::test]
async fn analyzes_erc20_transfer_to_unknown_recipient() {
    let payload = json!({
        "chain_id": 1,
        "from": "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        "to": "0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48",
        "value": "0",
        "data": "0xa9059cbb00000000000000000000000022222222222222222222222222222222222222220000000000000000000000000000000000000000000000000000000000000064"
    });

    let (status, body) = post_analyze(payload).await;

    assert_eq!(status, StatusCode::OK);

    // ERC-20 transfer +20, unknown recipient +20 = 40
    assert_eq!(body["risk_score"], 40);
    assert_eq!(body["risk_level"], "medium");
    assert_eq!(body["recommendation"], "review");
    assert_eq!(body["summary"], "ERC-20 token transfer detected");

    assert_eq!(body["decoded_action"]["type"], "erc20_transfer");
    assert_eq!(
        body["decoded_action"]["token"],
        "0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48"
    );
    assert_eq!(
        body["decoded_action"]["recipient"],
        "0x2222222222222222222222222222222222222222"
    );
    assert_eq!(
        body["decoded_action"]["amount"],
        "0x0000000000000000000000000000000000000000000000000000000000000064"
    );

    let codes = finding_codes(&body);
    assert!(codes.contains(&"ERC20_TRANSFER"));
    assert!(codes.contains(&"KNOWN_TOKEN"));
    assert!(codes.contains(&"UNKNOWN_RECIPIENT"));
}

#[tokio::test]
async fn analyzes_erc20_transfer_to_trusted_recipient() {
    let payload = json!({
        "chain_id": 1,
        "from": "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        "to": "0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48",
        "value": "0",
        "data": "0xa9059cbb00000000000000000000000099999999999999999999999999999999999999990000000000000000000000000000000000000000000000000000000000000064"
    });

    let (status, body) = post_analyze(payload).await;

    assert_eq!(status, StatusCode::OK);

    // ERC-20 transfer +20, trusted recipient -15 = 5
    assert_eq!(body["risk_score"], 5);
    assert_eq!(body["risk_level"], "low");
    assert_eq!(body["recommendation"], "allow");
    assert_eq!(body["summary"], "ERC-20 token transfer detected");

    assert_eq!(body["decoded_action"]["type"], "erc20_transfer");
    assert_eq!(
        body["decoded_action"]["recipient"],
        "0x9999999999999999999999999999999999999999"
    );

    let codes = finding_codes(&body);
    assert!(codes.contains(&"ERC20_TRANSFER"));
    assert!(codes.contains(&"KNOWN_TOKEN"));
    assert!(codes.contains(&"TRUSTED_RECIPIENT"));
}

#[tokio::test]
async fn analyzes_unknown_contract_call_to_unknown_target() {
    let payload = json!({
        "chain_id": 1,
        "from": "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        "to": "0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
        "value": "0",
        "data": "0xdeadbeef0000000000000000000000001111111111111111111111111111111111111111"
    });

    let (status, body) = post_analyze(payload).await;

    assert_eq!(status, StatusCode::OK);

    // Unknown calldata +40, unknown contract +25 = 65
    assert_eq!(body["risk_score"], 65);
    assert_eq!(body["risk_level"], "high");
    assert_eq!(body["recommendation"], "review");
    assert_eq!(body["summary"], "Unknown contract call detected");

    assert_eq!(body["decoded_action"]["type"], "unknown_call");
    assert_eq!(
        body["decoded_action"]["target"],
        "0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"
    );

    let codes = finding_codes(&body);
    assert!(codes.contains(&"UNKNOWN_CALLDATA"));
    assert!(codes.contains(&"UNKNOWN_CONTRACT"));
}

#[tokio::test]
async fn analyzes_unknown_contract_call_to_trusted_target() {
    let payload = json!({
        "chain_id": 1,
        "from": "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        "to": "0x7a250d5630b4cf539739df2c5dacb4c659f2488d",
        "value": "0",
        "data": "0xdeadbeef0000000000000000000000001111111111111111111111111111111111111111"
    });

    let (status, body) = post_analyze(payload).await;

    assert_eq!(status, StatusCode::OK);

    // Unknown calldata +40, trusted target -15 = 25
    assert_eq!(body["risk_score"], 25);
    assert_eq!(body["risk_level"], "medium");
    assert_eq!(body["recommendation"], "review");
    assert_eq!(body["summary"], "Unknown contract call detected");

    assert_eq!(body["decoded_action"]["type"], "unknown_call");
    assert_eq!(
        body["decoded_action"]["target"],
        "0x7a250d5630b4cf539739df2c5dacb4c659f2488d"
    );

    let codes = finding_codes(&body);
    assert!(codes.contains(&"UNKNOWN_CALLDATA"));
    assert!(codes.contains(&"TRUSTED_TARGET"));
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

    // Malformed approve selector becomes unknown call.
    // Unknown calldata +40, unknown contract +25 = 65
    assert_eq!(body["risk_score"], 65);
    assert_eq!(body["risk_level"], "high");
    assert_eq!(body["recommendation"], "review");

    assert_eq!(body["decoded_action"]["type"], "unknown_call");

    let codes = finding_codes(&body);
    assert!(codes.contains(&"UNKNOWN_CALLDATA"));
    assert!(codes.contains(&"UNKNOWN_CONTRACT"));
}

#[tokio::test]
async fn unlimited_approval_to_trusted_spender_is_less_risky_than_unknown_spender() {
    let unknown_spender_payload = json!({
        "chain_id": 1,
        "from": "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        "to": "0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48",
        "value": "0",
        "data": "0x095ea7b30000000000000000000000001111111111111111111111111111111111111111ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
    });

    let trusted_spender_payload = json!({
        "chain_id": 1,
        "from": "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        "to": "0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48",
        "value": "0",
        "data": "0x095ea7b30000000000000000000000007a250d5630b4cf539739df2c5dacb4c659f2488dffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
    });

    let (_, unknown_body) = post_analyze(unknown_spender_payload).await;
    let (_, trusted_body) = post_analyze(trusted_spender_payload).await;

    assert_eq!(unknown_body["risk_score"], 95);
    assert_eq!(trusted_body["risk_score"], 55);

    assert!(
        trusted_body["risk_score"].as_u64().unwrap() < unknown_body["risk_score"].as_u64().unwrap()
    );

    let unknown_codes = finding_codes(&unknown_body);
    let trusted_codes = finding_codes(&trusted_body);

    assert!(unknown_codes.contains(&"UNKNOWN_SPENDER"));
    assert!(trusted_codes.contains(&"TRUSTED_SPENDER"));
}
