use axum::extract::FromRef;
use store::DatabaseConnection;



#[derive(Clone)]
pub struct AppState{
    pub(crate) conn: DatabaseConnection,
}

impl FromRef<AppState> for DatabaseConnection {
    fn from_ref(state: &AppState) -> Self {
        state.conn.clone()
    }
}