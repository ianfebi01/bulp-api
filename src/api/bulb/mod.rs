pub mod dto;
pub mod handlers;
mod repo;

use deadpool_postgres::Pool;
use utoipa_axum::{router::OpenApiRouter, routes};

// Brings both the handler fns and the `__path_*` types that `#[utoipa::path]`
// generates alongside them into scope, which is what `routes!` needs.
use handlers::*;

/// Bulb routes plus their OpenAPI paths.
///
/// `routes!` reads the URL and method straight off each handler's
/// `#[utoipa::path]`, so there is no route string to keep in sync here.
/// Handlers sharing a URL must be listed in the same `routes!` call — that is
/// what turns `GET`/`PUT` on `/v1/bulb` into one method router.
///
/// Only `GET /bulb` is unversioned — it is the endpoint polled by deployed
/// IoT devices and must not move. Every other route lives under `/v1/`.
pub fn router() -> OpenApiRouter<Pool> {
    OpenApiRouter::new()
        .routes(routes!(get_bulb))
        .routes(routes!(get_bulb_v1, set_bulb))
        .routes(routes!(bulb_on))
        .routes(routes!(bulb_off))
}
