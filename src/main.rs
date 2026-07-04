mod db;
mod handlers;
mod models;

use axum::{routing::get, routing::post, Router};
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};

use crate::db::Db;
use crate::handlers::{bulb_off, bulb_on, get_bulb, set_bulb, AppState};

#[tokio::main]
async fn main() {
    // Open (or create) the SQLite database — path from env or default
    let db_path = std::env::var("BULB_DB_PATH").unwrap_or_else(|_| "bulb.db".into());
    let db = Db::open(&db_path).expect("Failed to open database");
    let state: AppState = Arc::new(db);

    // CORS — allow all origins for ESP32 access
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/bulb", get(get_bulb).put(set_bulb))
        .route("/bulb/on", post(bulb_on))
        .route("/bulb/off", post(bulb_off))
        .layer(cors)
        .with_state(state);

    let addr = "0.0.0.0:3000";
    println!("💡 Bulb API listening on http://{addr}", addr = addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
