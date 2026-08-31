//! Schedule resource — CRUD endpoints under `/v1/schedules`.
//!
//! Scheduler (cron) integration is intentionally not wired here yet: the
//! `tokio-cron-scheduler` runner is still commented out in `main.rs`. These
//! handlers only persist schedules; firing them is a follow-up.

pub mod dto;
pub mod handlers;
pub mod repo;

use deadpool_postgres::Pool;
use utoipa_axum::{router::OpenApiRouter, routes};

// Brings both the handler fns and the `__path_*` types that `#[utoipa::path]`
// generates alongside them into scope, which is what `routes!` needs.
use handlers::*;

/// Schedule routes plus their OpenAPI paths.
///
/// Handlers sharing a URL must be listed in the same `routes!` call — that is
/// what turns `POST`/`GET` on `/v1/schedules` (and `GET`/`PUT`/`DELETE` on
/// `/v1/schedules/{id}`) into one method router each.
pub fn router() -> OpenApiRouter<Pool> {
    OpenApiRouter::new()
        .routes(routes!(create_schedule, list_schedules))
        .routes(routes!(get_schedule, update_schedule, delete_schedule))
}
