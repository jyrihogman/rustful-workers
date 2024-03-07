use worker::{
    console_error, console_log, event, Context, Date, Env, Request, Response, Result, RouteContext,
    Router,
};

use api::qstash::{send_to_qstash, NotificationMessage};
use auth::{authenticate, authenticate_qstash_request, authorize};
use db::{get_all_notifications, get_user_subscribers};

mod api;
mod auth;
mod db;

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
                    console_error!("Error authenticating: {}", e);
                    Response::error(e, 403)
                }
            }
        })
        .post_async("/notifications", |req, ctx| async move {
            match authenticate(&req, &ctx.env).await {
                Ok(_) => handle_new_notification(req, ctx).await,
                Err(e) => {
                    console_error!("Error authenticating: {}", e);
                    Response::error(e, 403)
                }
            }
        })
        .post_async("/notifications/consume", |req, ctx| async move {
            match authenticate_qstash_request(&req, &ctx.env) {
                Ok(_) => handle_consume(req, ctx).await,
                Err(e) => Response::error(e, 403),
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

async fn handle_consume(mut req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let body = match req.json::<NotificationMessage>().await {
        Ok(body) => body,
        Err(e) => {
            console_error!("Error serializing request body: {}", e);
            return Response::error("Invalid Request Body", 400);
        }
    };

    let subscribers = match get_user_subscribers(body.user_id, &ctx.env).await {
        Ok(subscribers) => subscribers,
        Err(e) => {
            console_error!("Error searching for subscribers: {}", e);
            return Response::error("Failed to get subscribers from database", 500);
        }
    };

    Response::from_json(&subscribers.rows)
}

async fn handle_new_notification(mut req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let request_body = match req.json::<NotificationMessage>().await {
        Ok(body) => body,
        Err(e) => {
            console_error!("Failed to parse request body: {}", e);
            return Response::error("Invalid Request Body", 404);
        }
    };

    match send_to_qstash(request_body, ctx).await {
        Ok(body) => Response::from_json(&body),
        Err(e) => Response::error(e.to_string(), 500),
    }
}
