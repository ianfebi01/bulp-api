pub mod bulb;
pub mod response;
pub mod schedule;

use axum::response::IntoResponse;
use utoipa_axum::router::OpenApiRouter;

use crate::error::AppError;
use crate::state::AppState;

/// Compose every resource router into the public API surface.
///
/// Every route is served under `/v1/` except `GET /bulb`, which is polled by
/// deployed IoT devices and must not move — so the bulb router is merged at the
/// root rather than nested. New resources should be nested under the `v1`
/// prefix, e.g. `.nest("/v1/schedules", schedule::router())`, which prefixes
/// both the axum routes and the OpenAPI paths in one step.
pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .merge(bulb::router())
        .merge(schedule::router())
}

/// Fallback for unmatched routes — renders the standard error envelope.
pub async fn not_found() -> impl IntoResponse {
    AppError::NotFound("route not found".into())
}
