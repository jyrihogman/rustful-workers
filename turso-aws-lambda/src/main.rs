use db::get_messages;
use lambda_http::{run, service_fn, tracing, Body, Error, Request, Response};
mod db;

/// This is the main body for the function.
/// Write your code inside it.
/// There are some code example in the following URLs:
/// - https://github.com/awslabs/aws-lambda-rust-runtime/tree/main/examples
async fn function_handler(_event: Request) -> Result<Response<Body>, Error> {
    let messages = get_messages().await.unwrap();
    let users_json = serde_json::to_string(&messages)?;
    let resp = Response::builder()
        .status(200)
        .header("content-type", "text/html")
        .body(users_json.into())
        .map_err(Box::new)?;
    Ok(resp)
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing::init_default_subscriber();

    run(service_fn(function_handler)).await
}
