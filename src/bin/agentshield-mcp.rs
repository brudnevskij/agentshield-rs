use anyhow::Result;
use rmcp::{
    ServiceExt, handler::server::wrapper::Parameters, schemars, tool, tool_router, transport::stdio,
};
use serde::{Deserialize, Serialize};

const DEFAULT_AGENTSHIELD_API_URL: &str = "http://localhost:3000/analyze";

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct AnalyzeTransactionRiskParams {
    /// Blockchain chain id. Example: 1 for Ethereum mainnet.
    chain_id: u64,

    /// Sender address, 20-byte 0x-prefixed hex string.
    from: String,

    /// Transaction target address, 20-byte 0x-prefixed hex string.
    /// Use null for contract creation.
    to: Option<String>,

    /// Native token amount in wei as a decimal string.
    #[serde(default = "default_zero")]
    value: String,

    /// Transaction calldata as a 0x-prefixed hex string.
    #[serde(default = "default_calldata")]
    data: String,
}

#[derive(Debug, Serialize)]
struct AgentShieldApiRequest {
    chain_id: u64,
    from: String,
    to: Option<String>,
    value: String,
    data: Option<String>,
}

fn default_zero() -> String {
    "0".to_string()
}

fn default_calldata() -> String {
    "0x".to_string()
}

#[derive(Clone)]
struct AgentShieldMcp {
    http_client: reqwest::Client,
    api_url: String,
}

#[tool_router(server_handler)]
impl AgentShieldMcp {
    #[tool(
        name = "analyze_transaction_risk",
        description = "Analyze an unsigned blockchain transaction before an autonomous agent signs it. Returns AgentShield risk score, risk level, recommendation, decoded action, and findings."
    )]
    async fn analyze_transaction_risk(
        &self,
        Parameters(params): Parameters<AnalyzeTransactionRiskParams>,
    ) -> String {
        match self.forward_to_agentshield(params).await {
            Ok(json) => json,
            Err(err) => serde_json::json!({
                "error": err.to_string()
            })
            .to_string(),
        }
    }
}

impl AgentShieldMcp {
    async fn forward_to_agentshield(&self, params: AnalyzeTransactionRiskParams) -> Result<String> {
        validate_address("from", &params.from)?;

        if let Some(to) = &params.to {
            validate_address("to", to)?;
        }

        validate_decimal_string("value", &params.value)?;
        validate_hex_data(&params.data)?;

        let request = AgentShieldApiRequest {
            chain_id: params.chain_id,
            from: params.from.to_lowercase(),
            to: params.to.map(|addr| addr.to_lowercase()),
            value: params.value,
            data: Some(params.data.to_lowercase()),
        };

        let response = self
            .http_client
            .post(&self.api_url)
            .json(&request)
            .send()
            .await?;

        let status = response.status();
        let body = response.text().await?;

        if !status.is_success() {
            anyhow::bail!("AgentShield API returned {status}: {body}");
        }

        Ok(body)
    }
}

fn validate_address(field: &str, value: &str) -> Result<()> {
    let is_valid = value.len() == 42
        && value.starts_with("0x")
        && value[2..].chars().all(|c| c.is_ascii_hexdigit());

    if !is_valid {
        anyhow::bail!("{field} must be a 20-byte 0x-prefixed hex address");
    }

    Ok(())
}

fn validate_decimal_string(field: &str, value: &str) -> Result<()> {
    if value.is_empty() || !value.chars().all(|c| c.is_ascii_digit()) {
        anyhow::bail!("{field} must be a decimal integer string");
    }

    Ok(())
}

fn validate_hex_data(value: &str) -> Result<()> {
    let is_valid = value.starts_with("0x")
        && value.len() % 2 == 0
        && value[2..].chars().all(|c| c.is_ascii_hexdigit());

    if !is_valid {
        anyhow::bail!("data must be an even-length 0x-prefixed hex string");
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let api_url = std::env::var("AGENTSHIELD_API_URL")
        .unwrap_or_else(|_| DEFAULT_AGENTSHIELD_API_URL.to_string());

    eprintln!("[agentshield-mcp] starting stdio MCP server");
    eprintln!("[agentshield-mcp] forwarding to: {api_url}");

    let server = AgentShieldMcp {
        http_client: reqwest::Client::new(),
        api_url,
    };

    let service = server.serve(stdio()).await?;
    service.waiting().await?;

    Ok(())
}
