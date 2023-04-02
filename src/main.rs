use lambda_http::{run, service_fn, Body, Error, Request, RequestExt, Response};
use serde_json::json;

async fn basic_handler(event: Request) -> Result<Response<Body>, Error> {
    let method = event.method();
    let queryparams = event.query_string_parameters();
    let pathparams = event.path_parameters();
    let path = event.raw_http_path();

    let resp = Response::builder()
        .status(200)
        .header("content-type", "application/json")
        .body(
            json!({
                "method": method.to_string(),
                "path": path,
                "queryparams": queryparams,
                "pathparams": pathparams,
            })
            .to_string()
            .into(),
        )
        .map_err(Box::new)?;
    Ok(resp)
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        // disable printing the name of the module in every log line.
        .with_target(false)
        // disabling time is handy because CloudWatch will add the ingestion time.
        .without_time()
        .init();

    // run(service_fn(function_handler)).await
    run(service_fn(basic_handler)).await
}
