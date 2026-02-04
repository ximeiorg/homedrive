use std::net::SocketAddr;
use std::sync::Arc;

use axum::{http::Method, http::StatusCode, response::IntoResponse, Router};
use tracing::info;
use tower_http::cors::{Any, CorsLayer};

use crate::{route::routes, secret::load_jwt_secret, state::AppState};

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
    // 初始化 JWT 密钥
    let jwt_secret = load_jwt_secret();
    services::member::init_jwt_secret(jwt_secret);

    // 初始化存储
    let storage_config = services::StorageConfig {
        root: "uploads".to_string(),
        base_url: "/uploads".to_string(),
    };
    let storage: Arc<dyn services::StorageService> = Arc::new(services::LocalStorage::new(storage_config));

    let conn = store::connect_db("sqlite:homedrive.db?mode=rwc", false)
        .await
        .unwrap();
    let shared_state = AppState {
        conn,
        storage,
    };

    // CORS 配置
    let cors_layer = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_headers(Any);

    let app = Router::new()
        .nest("/api", routes(shared_state.clone()))
        .layer(cors_layer)
        .with_state(shared_state)
        .fallback(index_handler);

    serve(app, 2300).await;
}

async fn index_handler() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, "404 not Found")
}
