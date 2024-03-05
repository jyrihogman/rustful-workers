use serde::{Deserialize, Serialize};
use uuid::Uuid;
use worker::*;

#[derive(Deserialize, Serialize)]
pub struct NotificationMessage {
    pub user_id: Uuid,
    pub message: String,
    #[serde(rename = "type")]
    pub type_: String,
}

#[derive(Deserialize, Serialize)]
pub struct PublishResponse {
    #[serde(rename = "messageId")]
    message_id: String,
    url: String,
}

pub async fn send_to_qstash(
    body: NotificationMessage,
    ctx: RouteContext<()>,
) -> std::result::Result<Vec<PublishResponse>, reqwest::Error> {
    let qstash_url = ctx.var("QSTASH_URL").unwrap().to_string();
    let qstash_topic = ctx.var("QSTASH_TOPIC").unwrap().to_string();
    let qstash_token = ctx.var("QSTASH_TOKEN").unwrap().to_string();

    reqwest::Client::new()
        .post(format!("{}/{}", qstash_url, qstash_topic))
        .header("Content-Type", "application/json")
        .bearer_auth(qstash_token)
        .json(&body)
        .send()
        .await?
        .json::<Vec<PublishResponse>>()
        .await
}
