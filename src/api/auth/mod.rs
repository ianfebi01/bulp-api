pub mod dto;
pub mod handlers;
pub mod jwt;
pub mod middleware;
pub mod repo;

use utoipa_axum::{router::OpenApiRouter, routes};

use crate::state::AppState;
use handlers::*;

/// Auth routes plus their OpenAPI paths.
pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(register_user))
        .routes(routes!(login))
        .routes(routes!(refresh))
        .routes(routes!(logout))
        .routes(routes!(me))
}
