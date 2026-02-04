use crate::state::AppState;
use axum::{
    Router,
    http::Method,
    routing::{delete, get, post, put},
};
use tower_http::cors::{Any, CorsLayer};

use crate::handler::member::{
    check_username_exists, create_member, delete_member, get_member, get_member_by_username,
    list_members, update_member, update_member_avatar, update_member_password,
};

pub fn routes(state: AppState) -> axum::Router<AppState> {
    let member_routes = Router::new()
        .route("/", post(create_member))
        .route("/", get(list_members))
        .route("/{id}", get(get_member))
        .route("/{id}", put(update_member))
        .route("/{id}", delete(delete_member))
        .route("/username/{username}", get(get_member_by_username))
        .route("/username/{username}/exists", get(check_username_exists))
        .route("/{id}/avatar", put(update_member_avatar))
        .route("/{id}/password", put(update_member_password));

    Router::new()
        .nest("/api/members", member_routes)
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
                .allow_headers(Any),
        )
        .with_state(state)
}
