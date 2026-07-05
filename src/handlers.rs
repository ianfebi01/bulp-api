use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use std::sync::Arc;

use crate::db::Db;
use crate::models::{BulbState, CreateScheduleRequest, Schedule, SetBulbRequest, UpdateScheduleRequest};
use crate::scheduler::ScheduleRunner;

/// Shared application state passed to every handler.
#[derive(Clone)]
pub struct AppState {
    pub db: Arc<Db>,
    pub scheduler: Arc<ScheduleRunner>,
}

// ── Bulb handlers ───────────────────────────────────────────────────

/// GET /bulb — return current bulb state.
pub async fn get_bulb(State(state): State<AppState>) -> Result<Json<BulbState>, StatusCode> {
    state
        .db
        .get_state()
        .map(|(is_on, updated_at)| Json(BulbState { is_on, updated_at }))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

/// POST /bulb/on — turn the bulb on.
pub async fn bulb_on(State(state): State<AppState>) -> Result<Json<BulbState>, StatusCode> {
    state
        .db
        .set_state(true)
        .map(|(is_on, updated_at)| Json(BulbState { is_on, updated_at }))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

/// POST /bulb/off — turn the bulb off.
pub async fn bulb_off(State(state): State<AppState>) -> Result<Json<BulbState>, StatusCode> {
    state
        .db
        .set_state(false)
        .map(|(is_on, updated_at)| Json(BulbState { is_on, updated_at }))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

/// PUT /bulb — set bulb state via JSON body { "is_on": true/false }.
pub async fn set_bulb(
    State(state): State<AppState>,
    Json(body): Json<SetBulbRequest>,
) -> Result<Json<BulbState>, StatusCode> {
    state
        .db
        .set_state(body.is_on)
        .map(|(is_on, updated_at)| Json(BulbState { is_on, updated_at }))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

// ── Schedule handlers ───────────────────────────────────────────────

/// POST /schedules — create a new schedule and register its cron job.
pub async fn create_schedule(
    State(state): State<AppState>,
    Json(body): Json<CreateScheduleRequest>,
) -> Result<(StatusCode, Json<Schedule>), StatusCode> {
    // Validate action
    if body.action != "on" && body.action != "off" {
        return Err(StatusCode::BAD_REQUEST);
    }

    let schedule = state
        .db
        .create_schedule(&body.name, &body.cron_expr, &body.action)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Register the cron job
    state
        .scheduler
        .add_job(&schedule)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok((StatusCode::CREATED, Json(schedule)))
}

/// GET /schedules — list all schedules.
pub async fn list_schedules(
    State(state): State<AppState>,
) -> Result<Json<Vec<Schedule>>, StatusCode> {
    state
        .db
        .list_schedules()
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

/// GET /schedules/:id — get a single schedule.
pub async fn get_schedule(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Schedule>, StatusCode> {
    state
        .db
        .get_schedule(&id)
        .map(Json)
        .map_err(|_| StatusCode::NOT_FOUND)
}

/// PUT /schedules/:id — update a schedule and reload its cron job.
pub async fn update_schedule(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(body): Json<UpdateScheduleRequest>,
) -> Result<Json<Schedule>, StatusCode> {
    // Validate action if provided
    if let Some(ref action) = body.action {
        if action != "on" && action != "off" {
            return Err(StatusCode::BAD_REQUEST);
        }
    }

    let schedule = state
        .db
        .update_schedule(
            &id,
            body.name.as_deref(),
            body.cron_expr.as_deref(),
            body.action.as_deref(),
            body.enabled,
        )
        .map_err(|_| StatusCode::NOT_FOUND)?;

    // Reload the cron job (removes old, adds new if enabled)
    state
        .scheduler
        .reload_job(&schedule)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(schedule))
}

/// DELETE /schedules/:id — delete a schedule and remove its cron job.
pub async fn delete_schedule(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, StatusCode> {
    // Remove cron job first
    state
        .scheduler
        .remove_job(&id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let deleted = state
        .db
        .delete_schedule(&id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if deleted {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}
