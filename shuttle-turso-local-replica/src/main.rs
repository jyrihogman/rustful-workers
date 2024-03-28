use std::sync::Arc;

use axum::{extract::State, http::StatusCode, response::IntoResponse, routing::get, Json, Router};
use libsql::Connection;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Message {
    id: u32,
    target_id: String,
    message: String,
}

async fn get_posts(
    State(client): State<Arc<Connection>>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    let mut rows = match client
        .query("select id, target_id, message from message", ())
        .await
    {
        Ok(res) => res,
        Err(err) => return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(err.to_string()))),
    };

    let mut users = vec![];
    while let Some(row) = rows.next().unwrap() {
        users.push(Message {
            id: row.get::<u32>(0).unwrap(),
            target_id: row.get::<String>(1).unwrap(),
            message: row.get::<String>(2).unwrap(),
        });
    }

    Ok((StatusCode::OK, Json(users)))
}

#[shuttle_runtime::main]
async fn axum(
    #[shuttle_turso::Turso(
        addr = "libsql://hauskis-turso-jyrihogman.turso.io",
        token = "{secrets.TURSO_DB_TOKEN}"
    )]
    client: Connection,
) -> shuttle_axum::ShuttleAxum {
    let client = Arc::new(client);

    let router = Router::new().route("/", get(get_posts)).with_state(client);

    Ok(router.into())
}
