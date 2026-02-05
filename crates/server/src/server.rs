use std::net::SocketAddr;
use std::sync::Arc;

use axum::{http::Method, http::StatusCode, response::IntoResponse, Router};
use tracing::info;
use tower_http::cors::{Any, CorsLayer};

use crate::{config::AppConfig, route::routes, state::AppState};

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
    // 加载配置
    let config = AppConfig::load().expect("Failed to load configuration");

    // 获取 JWT 密钥
    let jwt_secret = config.jwt_secret();
    services::member::init_jwt_secret(jwt_secret);

    // 初始化存储
    let storage_config = services::StorageConfig {
        root: config.storage.volume.clone(),
    };
    let storage: Arc<dyn services::StorageService> = Arc::new(services::LocalStorage::new(storage_config));

    let conn = store::connect_db(&config.database.url, false)
        .await
        .unwrap();

    let shared_state = AppState {
        conn,
        storage,
        config: Arc::new(config),
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

    // 获取端口（优先使用环境变量）
    let port = crate::config::get_port_from_env()
        .unwrap_or_else(|| AppConfig::load().map(|c| c.server.port).unwrap_or(2300));

    serve(app, port).await;
}

async fn index_handler() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, "404 not Found")
}
