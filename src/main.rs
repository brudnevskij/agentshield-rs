use axum::{
    Json, Router,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Debug, Deserialize)]
struct AnalyzeRequest {
    chain_id: u64,
    from: String,
    to: Option<String>,
    value: String,
    data: Option<String>,
}

#[derive(Debug, Serialize)]
struct AnalyzeResponse {
    risk_score: u8,
    risk_level: RiskLevel,
    recommendation: Recommendation,
    summary: String,
    decoded_action: DecodedAction,
    findings: Vec<Finding>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
enum Recommendation {
    Allow,
    Review,
    Reject,
}

#[derive(Debug, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum DecodedAction {
    Erc20Approve {
        token: String,
        spender: String,
        amount: String,
    },
    Erc20Transfer {
        token: String,
        recipient: String,
        amount: String,
    },
    NativeTransfer {
        recipient: String,
        amount: String,
    },
    UnknownCall {
        target: Option<String>,
        calldata: Option<String>,
    },
}

#[derive(Debug, Serialize)]
struct Finding {
    code: String,
    severity: RiskLevel,
    score_impact: i32,
    title: String,
    description: String,
}

#[tokio::main]
async fn main() {
    init_tracing();

    let app = Router::new()
        .route("/", get(root))
        .route("/health", get(health))
        .route("/analyze", post(analyze))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("failed to bind server");

    println!("AgentShield running on http://localhost:3000");

    axum::serve(listener, app).await.expect("server failed");
}

fn init_tracing() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "agentshield_rs=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
}

async fn root() -> &'static str {
    "AgentShield API"
}

async fn health() -> &'static str {
    "ok"
}

async fn analyze(Json(req): Json<AnalyzeRequest>) -> impl IntoResponse {
    println!("Received analyze request: {:?}", req);

    let response = AnalyzeResponse {
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
    };

    (StatusCode::OK, Json(response))
}
