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
/// what turns `GET`/`PUT` on `/bulb` into one method router.
pub fn router() -> OpenApiRouter<Pool> {
    OpenApiRouter::new()
        .routes(routes!(get_bulb, set_bulb))
        .routes(routes!(get_bulb_v2))
        .routes(routes!(bulb_on))
        .routes(routes!(bulb_off))
}
