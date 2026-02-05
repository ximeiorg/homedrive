use std::net::SocketAddr;
use std::sync::Arc;

use axum::{Router, http::Method, http::StatusCode, response::IntoResponse};
use tower_http::cors::{Any, CorsLayer};
use tracing::info;

use crate::{config::AppConfig, route::routes, state::AppState};

// 配置 jsonwebtoken 的加密提供者
fn configure_jwt() {
    jsonwebtoken::crypto::rust_crypto::DEFAULT_PROVIDER
        .install_default()
        .unwrap();
}

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
    // 配置 JWT 加密提供者（必须在其他 JWT 操作之前调用）
    configure_jwt();

    // 加载配置
    let config = AppConfig::load().expect("Failed to load configuration");

    // 获取 JWT 密钥
    let jwt_secret = config.jwt_secret();
    services::member::init_jwt_secret(jwt_secret);

    // 初始化存储
    let storage_config = services::StorageConfig {
        root: config.storage.volume.clone(),
    };
    let storage: Arc<dyn services::StorageService> =
        Arc::new(services::LocalStorage::new(storage_config));

    let conn = store::connect_db(&config.database.url, false)
        .await
        .unwrap();

    let conn_arc = Arc::new(conn);

    // 创建任务 channel
    let (task_sender, _) = services::create_task_channel(100);

    // 创建任务工作器
    let task_config = services::TaskWorkerConfig {
        poll_interval_ms: 1000,
        max_concurrent: 10,
    };

    // 创建工作器并注册处理器
    let (mut worker, _) = services::TaskWorker::new(conn_arc.clone(), Some(task_config), 100);

    // 注册任务处理器
    worker.register_handler(Arc::new(services::SyncDirectoryHandler::new(
        config.storage.volume.clone(),
        conn_arc.clone(),
    )));

    // 在后台启动任务工作器
    tokio::spawn(async move {
        worker.start().await;
    });

    let shared_state = AppState {
        conn: (*conn_arc).clone(),
        storage,
        config: Arc::new(config),
        sync_task_sender: Arc::new(task_sender),
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
