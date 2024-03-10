use url::Url;
use worker::{
    console_error, console_log, event, Context, Date, Env, Error, Request, Response, Result,
    RouteContext, Router,
};

use libsql_client::{
    http::{Client, InnerClient},
    workers::HttpClient,
    Config, ResultSet,
};
use serde::{Deserialize, Serialize};

use kv_cache::{get_cache, set_cache};

use auth::{authenticate, authorize};

mod auth;

#[derive(Serialize, Deserialize)]
pub struct Message {
    id: String,
    target: String,
    message: String,
}

fn log_request(req: &Request) {
    console_log!(
        "{} - [{}], located at: {:?}, within: {}",
        Date::now().to_string(),
        req.path(),
        req.cf().coordinates().unwrap_or_default(),
        req.cf().region().unwrap_or("Unknown Region".into())
    );
}

#[event(fetch)]
pub async fn main(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    log_request(&req);

    Router::new()
        .get_async("/notifications", |req, ctx| async move {
            match authenticate(&req, &ctx.env)
                .await
                .and_then(|perms| authorize("read", perms))
            {
                Ok(_) => handle_get_notifications(req, ctx).await,
                Err(e) => {
                    console_error!("Error authenticating: {:?}", e);
                    Response::error(e, 403)
                }
            }
        })
        .run(req, env)
        .await
}

async fn handle_get_notifications(_req: Request, ctx: RouteContext<()>) -> Result<Response> {
    match get_all_notifications(&ctx.env).await {
        Ok(result_set) => Response::from_json(&result_set.rows),
        Err(e) => {
            console_error!("Error fetching messages from database: {}", e);
            Response::error(e.to_string(), 500)
        }
    }
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

pub async fn get_all_notifications(env: &Env) -> Result<ResultSet> {
    let client = create_client(env)?;

    if let Some(cached_result) = get_cache::<ResultSet>("messages_cache").await {
        return Ok(cached_result);
    }

    let results = client
        .execute("SELECT * FROM notifications")
        .await
        .map_err(|e| {
            console_error!("Error fetching messages from db");
            Error::from(e.to_string())
        })?;

    match set_cache("messages_cache", &results).await {
        Ok(_) => {}
        Err(e) => {
            console_error!("Error caching messages from db: {}", e);
        }
    }

    Ok(results)
}
