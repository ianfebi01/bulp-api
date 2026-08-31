use bulb_api::{api, db, openapi::ApiDoc};

use tower_http::cors::{Any, CorsLayer};
use utoipa::OpenApi;
use utoipa_axum::router::OpenApiRouter;
use utoipa_swagger_ui::SwaggerUi;

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

    // One router description, split into the thing that serves traffic and the
    // thing that documents it.
    let (routes, api_doc) = OpenApiRouter::with_openapi(ApiDoc::openapi())
        .merge(api::router())
        .split_for_parts();

    let app = routes
        .merge(SwaggerUi::new("/docs").url("/api-docs/openapi.json", api_doc))
        .layer(cors)
        .fallback(api::not_found)
        .with_state(pool);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("failed to bind port 3000");

    println!("listening on http://0.0.0.0:3000");
    println!("swagger ui on http://0.0.0.0:3000/docs");
    axum::serve(listener, app).await.expect("server error");
}
