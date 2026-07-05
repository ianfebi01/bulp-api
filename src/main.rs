mod db;
mod handlers;
mod models;
mod scheduler;

use axum::{
    routing::{get, post},
    Router,
};
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};

use crate::db::Db;
use crate::handlers::{
    bulb_off, bulb_on, create_schedule, delete_schedule, get_bulb, get_schedule, list_schedules,
    set_bulb, update_schedule, AppState,
};
use crate::scheduler::ScheduleRunner;

#[tokio::main]
async fn main() {
    // Structured logging
    tracing_subscriber::fmt::init();

    // Open (or create) the SQLite database — path from env or default
    let db_path = std::env::var("BULB_DB_PATH").unwrap_or_else(|_| "bulb.db".into());
    let db = Arc::new(Db::open(&db_path).expect("Failed to open database"));

    // ── Scheduler: crash recovery ───────────────────────────────────
    let runner = Arc::new(
        ScheduleRunner::new(Arc::clone(&db))
            .await
            .expect("Failed to create scheduler"),
    );

    // Reload all enabled schedules from DB (crash-recovery path).
    runner
        .load_all_from_db()
        .await
        .expect("Failed to load schedules from database");

    // Start the background cron loop.
    runner
        .start()
        .await
        .expect("Failed to start scheduler");

    tracing::info!("Scheduler started — cron jobs are running");

    let state = AppState {
        db,
        scheduler: runner,
    };

    // CORS — allow all origins for ESP32 access
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        // Bulb routes
        .route("/bulb", get(get_bulb).put(set_bulb))
        .route("/bulb/on", post(bulb_on))
        .route("/bulb/off", post(bulb_off))
        // Schedule routes
        .route("/schedules", get(list_schedules).post(create_schedule))
        .route(
            "/schedules/{id}",
            get(get_schedule).put(update_schedule).delete(delete_schedule),
        )
        .layer(cors)
        .with_state(state);

    let addr = "0.0.0.0:3000";
    tracing::info!("💡 Bulb API listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
