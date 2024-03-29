use db::{create_db, get_messages};
use std::sync::Arc;
use tokio::sync::Mutex;

use axum::{extract::State, http::StatusCode, response::IntoResponse, routing::get, Json, Router};
use libsql::Database;

mod db;

#[derive(Clone)]
struct AppState {
    db: Arc<Mutex<Database>>,
}

async fn handle_get_messages(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    match get_messages(state.db).await {
        Ok(messages) => Ok((StatusCode::OK, Json(messages))),
        Err(err) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(err.to_string()))),
    }
}

#[shuttle_runtime::main]
async fn axum() -> shuttle_axum::ShuttleAxum {
    let db = match create_db().await {
        Ok(db) => Arc::new(Mutex::new(db)),
        Err(e) => return Err(shuttle_runtime::Error::Database(e.to_string())),
    };

    let state = AppState { db };

    let router = Router::new()
        .route("/", get(handle_get_messages))
        .with_state(state);

    Ok(router.into())
}
