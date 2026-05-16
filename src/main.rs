mod handlers;
mod types;

use axum::{
    Router,
    routing::{get, post},
};
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::handlers::{analyze, health, root};

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
