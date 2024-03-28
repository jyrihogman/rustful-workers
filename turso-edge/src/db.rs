use std::u32;

use serde::{de::DeserializeOwned, Deserialize, Serialize};

use libsql_client::{
    de,
    http::{Client, InnerClient},
    workers::HttpClient,
    Config, ResultSet,
};
use url::Url;
use worker::*;

#[derive(Serialize, Deserialize)]
pub struct Message {
    id: u32,
    target_id: String,
    message: String,
}

fn result_set_to_json<T: DeserializeOwned>(result_set: ResultSet) -> Result<Vec<T>> {
    result_set
        .rows
        .iter()
        .map(de::from_row)
        .collect::<std::result::Result<Vec<T>, _>>()
        .map_err(|e| Error::from(e.to_string()))
}

fn create_config(env: &Env) -> Result<Config> {
    let url = Url::parse(&env.var("LIBSQL_CLIENT_URL")?.to_string())?;
    let auth_token = env.var("LIBSQL_CLIENT_TOKEN")?;

    Ok(Config {
        url,
        auth_token: Some(auth_token.to_string()),
    })
}

fn create_client(env: &Env) -> std::result::Result<Client, Error> {
    let config = create_config(env)?;
    Client::from_config(InnerClient::Workers(HttpClient), config)
        .map_err(|e| Error::from(e.to_string()))
}

pub async fn get_all_notifications(env: &Env) -> Result<Vec<Message>> {
    let client = create_client(env)?;

    client
        .execute("SELECT id, target_id, message FROM message")
        .await
        .map(result_set_to_json::<Message>)
        .map_err(|e| {
            console_error!("Error fetching messages from db");
            Error::from(e.to_string())
        })?
}
