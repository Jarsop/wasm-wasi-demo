use anyhow::Result;
use axum::{body::Bytes, http::StatusCode, response::IntoResponse, routing::get, Json, Router};

const ADDR: &str = "127.0.0.1:3000";

#[tokio::main]
async fn main() -> Result<()> {
    let app = Router::new().route(
        "/echo",
        get(echo)
            .post(echo_with_body)
            .put(echo_with_body)
            .delete(echo_with_body),
    );

    let listener = tokio::net::TcpListener::bind(ADDR).await?;
    println!("Listening on {}", ADDR);
    axum::serve(listener, app).await.map_err(Into::into)
}

async fn echo() -> impl IntoResponse {
    (StatusCode::OK, Json("Hello, World!"))
}

async fn echo_with_body(body: Bytes) -> impl IntoResponse {
    let mut response = String::from_utf8_lossy(&body);
    response.to_mut().push_str(" (echoed)");
    (StatusCode::OK, response.to_string())
}
