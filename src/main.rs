use bulb_api::{db, handlers};

use axum::{
    routing::{get, post},
    Router,
};

use tower_http::cors::{Any, CorsLayer};

use crate::handlers::{
    get_bulb,
    bulb_on,
    not_found,
};

#[tokio::main]
async fn main() {
    // Structured logging
    tracing_subscriber::fmt::init();

    dotenvy::dotenv().ok();

    let pool = db::create_pool();

    // // ── Scheduler: crash recovery ───────────────────────────────────
    // let runner = Arc::new(
    //     ScheduleRunner::new(Arc::clone(&db))
    //         .await
    //         .expect("Failed to create scheduler"),
    // );

    // // Reload all enabled schedules from DB (crash-recovery path).
    // runner
    //     .load_all_from_db()
    //     .await
    //     .expect("Failed to load schedules from database");

    // // Start the background cron loop.
    // runner
    //     .start()
    //     .await
    //     .expect("Failed to start scheduler");

    // tracing::info!("Scheduler started — cron jobs are running");

    // CORS — allow all origins for ESP32 access
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        // Bulb routes
        .route("/bulb", get(get_bulb))
        .route("/bulb/on", post(bulb_on))
        // .route("/bulb/off", post(bulb_off))
        // // Schedule routes
        // .route("/schedules", get(list_schedules).post(create_schedule))
        // .route(
        //     "/schedules/{id}",
        //     get(get_schedule).put(update_schedule).delete(delete_schedule),
        // )
        .layer(cors)
        .fallback(not_found)
        .with_state(pool);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("failed to bind port 3000");

    println!("listening on http://0.0.0.0:3000");
    axum::serve(listener, app).await.expect("server error");
}
