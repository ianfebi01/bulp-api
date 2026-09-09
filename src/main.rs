use std::sync::Arc;

use bulb_api::{api, api::schedule::runner::ScheduleRunner, db, openapi::ApiDoc, state::AppState};

use tower_http::cors::{Any, CorsLayer};
use utoipa::OpenApi;
use utoipa_axum::router::OpenApiRouter;
use utoipa_swagger_ui::SwaggerUi;

#[tokio::main]
async fn main() {
    // Structured logging. Set RUST_LOG=info to see the scheduler's output.
    tracing_subscriber::fmt::init();

    dotenvy::dotenv().ok();

    let pool = db::create_pool();

    // ── Scheduler ───────────────────────────────────────────────────
    // Built once, before the server, and shared with every request through
    // `AppState`. `start()` spawns the background tick loop, so nothing here
    // needs a `loop { sleep }` of its own — the tasks run alongside axum.
    let runner = ScheduleRunner::start(pool.clone())
        .await
        .expect("failed to start scheduler");

    // Crash recovery: jobs live only in memory, so rebuild them from the
    // database on every boot. Fatal on purpose — booting with an unknown set of
    // schedules is worse than not booting at all.
    let loaded = runner
        .load_from_db()
        .await
        .expect("failed to load schedules from database");
    tracing::info!(
        "scheduler started with {loaded} enabled schedule(s); cron expressions \
         are read in {}",
        runner.timezone()
    );

    let state = AppState {
        pool,
        runner: Arc::clone(&runner),
    };

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
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("failed to bind port 3000");

    tracing::info!("listening on http://0.0.0.0:3000");
    tracing::info!("swagger ui on http://0.0.0.0:3000/docs");
    axum::serve(listener, app).await.expect("server error");
}
