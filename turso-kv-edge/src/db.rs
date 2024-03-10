use libsql_client::{
    http::{Client, InnerClient},
    workers::HttpClient,
    Config, ResultSet, Statement,
};
use url::Url;
use uuid::Uuid;
use worker::*;

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

pub async fn get_user_subscribers(user_id: Uuid, env: &Env) -> Result<ResultSet> {
    let client = create_client(env)?;

    client
        .execute(Statement::with_args(
            "SELECT * FROM user_subscribers WHERE user_id = ?1",
            &[user_id.to_string()],
        ))
        .await
        .map_err(|e| worker::Error::from(e.to_string()))
}

pub async fn get_all_notifications(env: &Env) -> Result<ResultSet> {
    let client = create_client(env)?;

    client
        .execute("SELECT * FROM notifications")
        .await
        .map_err(|e| {
            console_error!("Error fetching messages from db");
            Error::from(e.to_string())
        })
}
