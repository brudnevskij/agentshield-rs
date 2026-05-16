pub mod decoder;
pub mod handlers;
pub mod registry;
pub mod risk;
pub mod types;

use crate::handlers::{analyze, health, root};
use alloy_primitives::Address;
use axum::{
    Router,
    routing::{get, post},
};
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use x402_axum::X402Middleware;
use x402_chain_eip155::{KnownNetworkEip155, V2Eip155Exact};
use x402_types::networks::USDC;

pub fn build_app() -> Router {
    let facilitator_url = std::env::var("X402_FACILITATOR_URL")
        .unwrap_or_else(|_| "http://127.0.0.1:8080".to_string());

    let pay_to: Address = std::env::var("X402_PAY_TO")
        .unwrap_or_else(|_| "0x0000000000000000000000000000000000000000".to_string())
        .parse()
        .expect("invalid X402_PAY_TO address");

    let x402 = X402Middleware::new(&facilitator_url);

    Router::new()
        .route("/", get(root))
        .route("/health", get(health))
        .route("/analyze", post(analyze))
        .route(
            "/x402/analyze",
            post(analyze).layer(x402.with_price_tag(V2Eip155Exact::price_tag(
                pay_to,
                USDC::base_sepolia().amount(10u64),
            ))),
        )
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
}
