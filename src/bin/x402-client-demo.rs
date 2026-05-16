use alloy_signer_local::PrivateKeySigner;
use anyhow::Result;
use reqwest::Client;
use serde_json::json;
use std::sync::Arc;
use x402_chain_eip155::V2Eip155ExactClient;
use x402_reqwest::{ReqwestWithPayments, ReqwestWithPaymentsBuild, X402Client};

const DEFAULT_URL: &str = "http://127.0.0.1:3000/x402/analyze";

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();

    let url = std::env::var("AGENTSHIELD_X402_URL").unwrap_or_else(|_| DEFAULT_URL.to_string());

    let private_key =
        std::env::var("X402_CLIENT_PRIVATE_KEY").expect("X402_CLIENT_PRIVATE_KEY must be set");

    let signer: Arc<PrivateKeySigner> = Arc::new(private_key.parse()?);

    let x402_client = X402Client::new().register(V2Eip155ExactClient::new(signer));

    let client = Client::new().with_payments(x402_client).build();

    let payload = json!({
        "chain_id": 1,
        "from": "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        "to": "0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48",
        "value": "0",
        "data": "0x095ea7b30000000000000000000000001111111111111111111111111111111111111111ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
    });

    let response = client
        .post(&url)
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .body(serde_json::to_vec(&payload)?)
        .send()
        .await?;

    let status = response.status();
    let body = response.text().await?;

    println!("Status: {status}");
    println!("{body}");

    Ok(())
}
