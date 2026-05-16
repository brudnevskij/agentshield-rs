use axum::{Json, http::StatusCode, response::IntoResponse};

use crate::{decoder::decode_transaction, risk::analyze_risk, types::AnalyzeRequest};

pub async fn root() -> &'static str {
    "AgentShield API"
}

pub async fn health() -> &'static str {
    "ok"
}

pub async fn analyze(Json(req): Json<AnalyzeRequest>) -> impl IntoResponse {
    tracing::info!(?req, "received analyze request");

    let decoded_action = decode_transaction(&req);
    let response = analyze_risk(decoded_action);

    (StatusCode::OK, Json(response))
}
