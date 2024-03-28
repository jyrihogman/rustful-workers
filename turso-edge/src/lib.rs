use std::future::Future;

use worker::{
    console_error, console_log, event, Context, Date, Env, Request, Response, Result, RouteContext,
    Router,
};

use api::qstash::{send_to_qstash, NotificationMessage};
use db::get_all_notifications;

use auth::authenticate;

mod api;
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
            handle_get_notifications(req, ctx).await
        })
        .post_async("/notifications", |req, ctx| async move {
            handle_route_with_authentication(req, ctx, handle_new_notification).await
        })
        .run(req, env)
        .await
}

async fn handle_get_notifications(_req: Request, ctx: RouteContext<()>) -> Result<Response> {
    match get_all_notifications(&ctx.env).await {
        Ok(messages) => Response::from_json(&messages),
        Err(e) => {
            console_error!("Error fetching messages from database: {}", e);
            Response::error(e.to_string(), 500)
        }
    }
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

async fn handle_route_with_authentication<F, Fut>(
    req: Request,
    ctx: RouteContext<()>,
    handler: F,
) -> Result<Response>
where
    F: Fn(Request, RouteContext<()>) -> Fut,
    Fut: Future<Output = Result<Response>>,
{
    match authenticate(&req, &ctx.env).await {
        Ok(_) => (),
        Err(e) => return Response::error(e, 401),
    };

    handler(req, ctx).await
}
