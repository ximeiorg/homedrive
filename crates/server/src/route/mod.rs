use crate::auth::auth_middleware;
use crate::state::AppState;
use axum::{
    Router,
    http::Method,
    routing::{delete, get, post, put},
};
use tower_http::cors::{Any, CorsLayer};

use crate::handler::member::{
    check_username_exists, create_member, delete_member, get_member, get_member_by_username,
    list_members, login, update_member, update_member_avatar, update_member_password,
};

pub fn routes(state: AppState) -> axum::Router<AppState> {
    // 公开路由 - 无需认证
    let public_routes = Router::new()
        .route("/login", post(login))
        .route("/username/{username}/exists", get(check_username_exists));

    // 受保护路由 - 需要 JWT 认证
    let protected_routes = Router::new()
        .route("/", post(create_member))
        .route("/", get(list_members))
        .route("/{id}", get(get_member))
        .route("/{id}", put(update_member))
        .route("/{id}", delete(delete_member))
        .route("/username/{username}", get(get_member_by_username))
        .route("/{id}/avatar", put(update_member_avatar))
        .route("/{id}/password", put(update_member_password))
        .route_layer(axum::middleware::from_fn(auth_middleware));

    Router::new()
        .nest("/api/members", public_routes.merge(protected_routes))
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
                .allow_headers(Any),
        )
        .with_state(state)
}
