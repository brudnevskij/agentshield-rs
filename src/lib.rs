pub mod decoder;
pub mod handlers;
pub mod types;

use axum::{
    Router,
    routing::{get, post},
};
use tower_http::{cors::CorsLayer, trace::TraceLayer};

use crate::handlers::{analyze, health, root};

pub fn build_app() -> Router {
    Router::new()
        .route("/", get(root))
        .route("/health", get(health))
        .route("/analyze", post(analyze))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
}
