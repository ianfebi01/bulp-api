pub mod dto;
pub mod handlers;
pub mod repo;

use utoipa_axum::{router::OpenApiRouter, routes};

use crate::state::AppState;

// Brings both the handler fns and the `__path_*` types that `#[utoipa::path]`
// generates alongside them into scope, which is what `routes!` needs.
use handlers::*;

/// Auth routes plus their OpenAPI paths.
///
/// Handlers sharing a URL must be listed in the same `routes!` call — that is
/// what turns `POST`/`GET` on `/v1/auth` (and `GET`/`PUT`/`DELETE` on
/// `/v1/auth/{id}`) into one method router each.
pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(register_user))
}
