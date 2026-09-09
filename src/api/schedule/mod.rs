//! Schedule resource — CRUD endpoints under `/v1/schedules`, plus the cron
//! runner that actually fires them.
//!
//! Persistence and execution are deliberately separate: [`repo`] owns the
//! `schedules` table, [`runner`] owns the in-memory cron jobs, and [`handlers`]
//! is the only place that touches both — it writes the row, then asks the
//! runner to match it.

pub mod dto;
pub mod handlers;
pub mod repo;
pub mod runner;

use utoipa_axum::{router::OpenApiRouter, routes};

use crate::state::AppState;

// Brings both the handler fns and the `__path_*` types that `#[utoipa::path]`
// generates alongside them into scope, which is what `routes!` needs.
use handlers::*;

/// Schedule routes plus their OpenAPI paths.
///
/// Handlers sharing a URL must be listed in the same `routes!` call — that is
/// what turns `POST`/`GET` on `/v1/schedules` (and `GET`/`PUT`/`DELETE` on
/// `/v1/schedules/{id}`) into one method router each.
pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(create_schedule, list_schedules))
        .routes(routes!(get_schedule, update_schedule, delete_schedule))
}
