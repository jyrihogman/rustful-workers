use std::{env, sync::Arc};

use libsql::{de::from_row, Builder, Database};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

#[derive(Serialize, Deserialize)]
pub struct Message {
    id: u32,
    target_id: String,
    message: String,
}

pub async fn create_db() -> Result<Database, libsql::Error> {
    let url = env::var("TURSO_URL").unwrap();
    let auth_token = env::var("TURSO_AUTH_TOKEN").unwrap();

    Builder::new_remote_replica("local.db", url, auth_token)
        .build()
        .await
}

pub async fn get_messages(client: Arc<Mutex<Database>>) -> Result<Vec<Message>, libsql::Error> {
    let mut rows = client
        .lock()
        .await
        .connect()?
        .query("SELECT id, target_id, message FROM message", ())
        .await?;

    let mut users = vec![];
    while let Some(row) = rows.next().await? {
        users.push(from_row::<Message>(&row).unwrap());
    }

    Ok(users)
}
