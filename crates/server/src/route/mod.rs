use axum::{Router, http::Method};
use crate::state::AppState;
use tower_http::cors::{Any, CorsLayer};
pub fn routes(_state: AppState) -> axum::Router<AppState> {
    Router::new().layer(
        CorsLayer::new()
            .allow_origin(Any)
            .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
            .allow_headers(Any),
    )
}
