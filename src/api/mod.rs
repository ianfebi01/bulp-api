pub mod auth;
pub mod bulb;
pub mod response;
pub mod schedule;

use axum::response::IntoResponse;
use utoipa_axum::router::OpenApiRouter;

use crate::api::auth::middleware::auth_middleware;
use crate::error::AppError;
use crate::state::AppState;

/// Compose every resource router into the public API surface.
///
/// Authentication middleware is applied to protect versioned bulb (`/v1/bulb/*`)
/// and schedule (`/v1/schedules/*`) routes.
///
/// `GET /bulb` remains unauthenticated so legacy/deployed IoT devices can poll it.
/// Auth routes (`/v1/auth/*`) remain public (with `/me` enforcing authentication via extractor).
pub fn router() -> OpenApiRouter<AppState> {
    let protected = OpenApiRouter::new()
        .merge(bulb::protected_router())
        .merge(schedule::router())
        .route_layer(axum::middleware::from_fn(auth_middleware));

    OpenApiRouter::new()
        .merge(bulb::public_router())
        .merge(auth::router())
        .merge(protected)
}

/// Fallback for unmatched routes — renders the standard error envelope.
pub async fn not_found() -> impl IntoResponse {
    AppError::NotFound("route not found".into())
}
