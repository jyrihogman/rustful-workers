use crate::api::qstash::{send_to_qstash, PublishRequestBody, PublishResponse};
use worker::*;

pub async fn publish_notification(
    mut req: Request,
    ctx: RouteContext<()>,
) -> std::result::Result<Vec<PublishResponse>, Error> {
    let body_result = req.json::<PublishRequestBody>().await;

    let body = match body_result {
        Ok(body) => body,
        Err(e) => {
            eprintln!("Error serializing request body: {}", e);
            return Err(e);
        }
    };

    match send_to_qstash(body, ctx).await {
        Ok(body) => Ok(body),
        Err(e) => Err(Error::from(e.to_string())),
    }
}
