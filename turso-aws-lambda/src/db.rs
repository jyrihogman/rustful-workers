use libsql::{de, Builder};
use serde::Serialize;

use std::env;

#[derive(Debug, serde::Deserialize, Serialize)]
    #[allow(dead_code)]
    pub struct User {
        id: String,
        id_hash: String,
        display_name: String,
    }


pub async fn get_messages() -> Result<Vec<User>, libsql::Error> {
    let url = env::var("LIBSQL_URL").expect("DB_URL env variable not set");
    let auth_token = env::var("LIBSQL_AUTH_TOKEN").expect("DB_TOKEN env variable not set");

    let db = Builder::new_remote(url, auth_token).build().await.unwrap();
    let mut rows = db.connect()?.query("SELECT * FROM users", ()).await?;
    let mut users = Vec::new();
    while let Some(row) = rows.next().await.unwrap() {
        let user = de::from_row::<User>(&row).unwrap();
        users.push(user);
    }
    Ok(users)
}