pub mod bulb;
pub mod response;
pub mod schedule;

use axum::response::IntoResponse;
use deadpool_postgres::Pool;
use utoipa_axum::router::OpenApiRouter;

use crate::error::AppError;

/// Compose every resource router into the public API surface.
///
/// Resource modules are merged at the root because the v1 URLs (`/bulb`,
/// `/bulb/on`, …) are consumed by deployed IoT devices and must not move. A
/// new resource with a clean prefix should be nested instead, e.g.
/// `.nest("/schedules", schedule::router())`, which prefixes both the axum
/// routes and the OpenAPI paths in one step.
pub fn router() -> OpenApiRouter<Pool> {
    OpenApiRouter::new().merge(bulb::router())
}

/// Fallback for unmatched routes — renders the standard error envelope.
pub async fn not_found() -> impl IntoResponse {
    AppError::NotFound("route not found".into())
}
