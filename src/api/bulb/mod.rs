pub mod dto;
pub mod handlers;
// `pub(crate)` rather than private: the schedule runner writes bulb state
// when a cron job fires, and it goes through this repo like everything else.
pub(crate) mod repo;

use utoipa_axum::{router::OpenApiRouter, routes};

use crate::state::AppState;

// Brings both the handler fns and the `__path_*` types that `#[utoipa::path]`
// generates alongside them into scope, which is what `routes!` needs.
use handlers::*;

/// Public unversioned bulb routes (`GET /bulb`) polled by IoT devices.
pub fn public_router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new().routes(routes!(get_bulb))
}

/// Protected `/v1/bulb` routes requiring authentication.
pub fn protected_router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(get_bulb_v1, set_bulb))
        .routes(routes!(bulb_on))
        .routes(routes!(bulb_off))
}

/// All bulb routes combined.
pub fn router() -> OpenApiRouter<AppState> {
    public_router().merge(protected_router())
}
