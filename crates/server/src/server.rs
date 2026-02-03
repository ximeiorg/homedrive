use std::net::SocketAddr;

use axum::{Router, http::StatusCode, response::IntoResponse};
use tracing::info;

use crate::{route::routes, state::AppState};

pub async fn serve(app: Router, port: u16) {
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    info!("server listening on {}, see: http://{}", addr, addr);
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    // .with_graceful_shutdown(shutdown_signal())
    .await
    .unwrap();
}

pub async fn start() {
    let conn = store::connect_db("sqlite:homedrive.db?mode=rwc", true)
        .await
        .unwrap();
    let shared_state = AppState { conn };

    let app = Router::new()
        .nest("/api", routes(shared_state.clone()))
        .with_state(shared_state)
        .fallback(index_handler);

    serve(app, 2300).await;
}

async fn index_handler() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, "404 not Found")
}
