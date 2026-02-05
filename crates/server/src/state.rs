use std::sync::Arc;

use axum::extract::FromRef;
use services::StorageService;
use store::DatabaseConnection;

use crate::config::AppConfig;

#[derive(Clone)]
pub struct AppState {
    pub(crate) conn: DatabaseConnection,
    pub(crate) storage: Arc<dyn StorageService>,
    pub(crate) config: Arc<AppConfig>,
    pub(crate) sync_task_sender: services::TaskSenderType,
}

impl FromRef<AppState> for DatabaseConnection {
    fn from_ref(state: &AppState) -> Self {
        state.conn.clone()
    }
}

impl FromRef<AppState> for Arc<dyn StorageService> {
    fn from_ref(state: &AppState) -> Self {
        state.storage.clone()
    }
}

impl FromRef<AppState> for Arc<AppConfig> {
    fn from_ref(state: &AppState) -> Self {
        state.config.clone()
    }
}

impl FromRef<AppState> for services::TaskSenderType {
    fn from_ref(state: &AppState) -> Self {
        state.sync_task_sender.clone()
    }
}
