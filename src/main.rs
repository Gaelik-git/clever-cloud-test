use std::env;

use axum::{http::StatusCode, response::IntoResponse, routing::get, Router};

#[tokio::main]
async fn main() {
    let port = env::var("PORT").unwrap_or("8080".to_owned());

    let app = Router::new().route("/health", get(health_check));
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{port}"))
        .await
        .unwrap();

    println!("Starting server on port {port}");
    axum::serve(listener, app).await.unwrap();
}

async fn health_check() -> impl IntoResponse {
    println!("health check");
    (StatusCode::OK, "OK")
}
