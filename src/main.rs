use std::env;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Router,
};
use fred::{
    error::ErrorKind,
    prelude::{ClientLike, Config, KeysInterface},
    types::Builder,
};

#[derive(Clone)]
struct AppState {
    materia_kv_pool: fred::clients::Pool,
}

#[tokio::main]
async fn main() {
    let port = env::var("PORT").unwrap_or("8080".to_owned());

    let kv_url = env::var("REDIS_URL").expect("REDIS_URL must be set");
    let config = Config::from_url(&kv_url).expect("Could create the fred config");

    let materia_kv_pool = Builder::from_config(config)
        .build_pool(2)
        .expect("Could create the pool");

    materia_kv_pool.init().await.expect("Could init the pool");

    let app_state = AppState { materia_kv_pool };

    let app = Router::new()
        .route("/health", get(health_check))
        .route("/ping", get(ping))
        .route("/key/{key}", get(get_key_value))
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{port}"))
        .await
        .unwrap();

    println!("Starting server on port {port}");
    axum::serve(listener, app).await.unwrap();
}

async fn ping(State(app_state): State<AppState>) -> impl IntoResponse {
    let res: String = app_state.materia_kv_pool.next().ping(None).await.unwrap();
    res
}

async fn health_check() -> impl IntoResponse {
    println!("health check");
    (StatusCode::OK, "OK")
}

async fn get_key_value(
    Path(key): Path<String>,
    State(app_state): State<AppState>,
) -> Result<String, StatusCode> {
    let client = app_state.materia_kv_pool.next();
    let val = client.get(key).await;

    val.map_err(|err| {
        if err.kind() == &ErrorKind::NotFound {
            StatusCode::NOT_FOUND
        } else {
            StatusCode::INTERNAL_SERVER_ERROR
        }
    })
}
