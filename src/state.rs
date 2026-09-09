//! Shared application state.
//!
//! Axum gives every handler one state value, but this app has two things worth
//! sharing: the Postgres pool and the cron runner. `AppState` bundles them, and
//! the `FromRef` impls below let each handler extract only the part it needs —
//! `State<Pool>` in the bulb handlers, `State<Arc<ScheduleRunner>>` in the
//! schedule ones. No handler has to know about the fields it does not use.
//!
//! Cloning is cheap on purpose: a `Pool` and an `Arc` are handles, so cloning
//! them per request copies a pointer, not the pool or the scheduler.

use std::sync::Arc;

use axum::extract::FromRef;
use deadpool_postgres::Pool;

use crate::api::schedule::runner::ScheduleRunner;

#[derive(Clone)]
pub struct AppState {
    pub pool: Pool,
    pub runner: Arc<ScheduleRunner>,
}

impl FromRef<AppState> for Pool {
    fn from_ref(state: &AppState) -> Self {
        state.pool.clone()
    }
}

impl FromRef<AppState> for Arc<ScheduleRunner> {
    fn from_ref(state: &AppState) -> Self {
        Arc::clone(&state.runner)
    }
}
