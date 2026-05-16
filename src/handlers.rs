use axum::{Json, http::StatusCode, response::IntoResponse};

use crate::types::{
    AnalyzeRequest, AnalyzeResponse, DecodedAction, Finding, Recommendation, RiskLevel,
};

pub async fn root() -> &'static str {
    "AgentShield API"
}

pub async fn health() -> &'static str {
    "ok"
}

pub async fn analyze(Json(req): Json<AnalyzeRequest>) -> impl IntoResponse {
    tracing::info!(?req, "received analyze request");

    let response = build_static_response(req);

    (StatusCode::OK, Json(response))
}

fn build_static_response(req: AnalyzeRequest) -> AnalyzeResponse {
    AnalyzeResponse {
        risk_score: 95,
        risk_level: RiskLevel::Critical,
        recommendation: Recommendation::Reject,
        summary: "Unlimited ERC-20 approval to unknown spender".to_string(),
        decoded_action: DecodedAction::Erc20Approve {
            token: req.to.unwrap_or_else(|| "0xUnknownToken".to_string()),
            spender: "0x1111111111111111111111111111111111111111".to_string(),
            amount: "unlimited".to_string(),
        },
        findings: vec![
            Finding {
                code: "ERC20_APPROVAL".to_string(),
                severity: RiskLevel::High,
                score_impact: 20,
                title: "ERC-20 approval detected".to_string(),
                description: "The transaction grants another address permission to spend tokens."
                    .to_string(),
            },
            Finding {
                code: "UNLIMITED_APPROVAL".to_string(),
                severity: RiskLevel::Critical,
                score_impact: 50,
                title: "Unlimited token approval".to_string(),
                description: "The spender can move all current and future token balance."
                    .to_string(),
            },
            Finding {
                code: "UNKNOWN_SPENDER".to_string(),
                severity: RiskLevel::High,
                score_impact: 25,
                title: "Unknown spender".to_string(),
                description: "The spender is not in the trusted address registry.".to_string(),
            },
        ],
    }
}
