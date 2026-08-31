//! Schedule resource — **not wired into [`crate::api::router`] yet.**
//!
//! The handlers below are the pre-Postgres implementation, kept commented out
//! until the scheduler is ported. To revive: uncomment, port the queries into a
//! `repo.rs` like [`crate::api::bulb`], add `#[utoipa::path]` to each handler,
//! then add a `router()` here and `.nest("/schedules", schedule::router())` in
//! `api::router()`.

pub mod dto;
pub mod handlers;
