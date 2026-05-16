use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct AnalyzeRequest {
    pub chain_id: u64,
    pub from: String,
    pub to: Option<String>,
    pub value: String,
    pub data: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct AnalyzeResponse {
    pub risk_score: u8,
    pub risk_level: RiskLevel,
    pub recommendation: Recommendation,
    pub summary: String,
    pub decoded_action: DecodedAction,
    pub findings: Vec<Finding>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Recommendation {
    Allow,
    Review,
    Reject,
}

#[derive(Debug, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum DecodedAction {
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
pub struct Finding {
    pub code: String,
    pub severity: RiskLevel,
    pub score_impact: i32,
    pub title: String,
    pub description: String,
}
