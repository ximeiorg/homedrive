use axum::extract::FromRef;
use services::StorageService;
use store::DatabaseConnection;

#[derive(Clone)]
pub struct AppState {
    pub(crate) conn: DatabaseConnection,
    pub(crate) storage: std::sync::Arc<dyn StorageService>,
}

impl FromRef<AppState> for DatabaseConnection {
    fn from_ref(state: &AppState) -> Self {
        state.conn.clone()
    }
}

impl FromRef<AppState> for std::sync::Arc<dyn StorageService> {
    fn from_ref(state: &AppState) -> Self {
        state.storage.clone()
    }
}
