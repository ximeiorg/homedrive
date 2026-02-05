use crate::auth::auth_middleware;
use crate::state::AppState;
use axum::{
    Router,
    routing::{delete, get, post, put},
};

use crate::handler::member::{
    check_members_empty, check_username_exists, create_member, delete_member, get_member,
    get_member_by_username, init_admin, list_members, login, update_member, update_member_avatar,
    update_member_password,
};

use crate::handler::file::{
    check_file_hash_exists,
    serve_file,
    upload_file,
};

pub fn routes(state: AppState) -> axum::Router<AppState> {
    // Public routes - no authentication required
    let public_routes = Router::new()
        .route("/login", post(login))
        .route("/username/{username}/exists", get(check_username_exists))
        .route("/empty", get(check_members_empty))
        .route("/init", post(init_admin));

    // File routes - check hash is public, upload and serve require auth
    let file_routes = Router::new()
        .route("/check-hash", get(check_file_hash_exists))
        .route("/upload", post(upload_file).layer(axum::middleware::from_fn(auth_middleware)))
        .route("/:storage_tag/*path", get(serve_file).layer(axum::middleware::from_fn(auth_middleware)));

    // Protected routes - require JWT authentication
    let protected_routes = Router::new()
        .route("/", post(create_member))
        .route("/", get(list_members))
        .route("/{id}", get(get_member))
        .route("/{id}", put(update_member))
        .route("/{id}", delete(delete_member))
        .route("/username/{username}", get(get_member_by_username))
        .route("/{id}/avatar", put(update_member_avatar))
        .route("/{id}/password", put(update_member_password))
        .layer(axum::middleware::from_fn(auth_middleware));

    Router::new()
        .nest("/members", public_routes.merge(protected_routes))
        .nest("/files", file_routes)
        .with_state(state)
}
