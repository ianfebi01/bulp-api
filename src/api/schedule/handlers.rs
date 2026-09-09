//! Schedule HTTP layer: extract, delegate to [`super::repo`], keep the cron
//! runner in step, wrap the response.
//!
//! The ordering inside each handler matters:
//!   1. validate (action, cron expression) — cheap, and rejects before we write
//!   2. write to Postgres — the source of truth
//!   3. tell the runner about it — the in-memory jobs follow the database
//!
//! Validating first means an invalid cron expression can never be persisted;
//! syncing last means the jobs always describe rows that really exist.

use std::sync::Arc;

use axum::{
    extract::{Path, State},
    Json,
};
use deadpool_postgres::Pool;
use uuid::Uuid;

use super::dto::{Action, CreateScheduleRequest, Schedule, UpdateScheduleRequest};
use super::repo;
use super::runner::ScheduleRunner;
use crate::api::response::ApiResponse;
use crate::error::AppError;

/// Parse a path id into a `Uuid`, reporting failure in the standard error
/// envelope.
///
/// `Path<Uuid>` would parse it for us, but axum's own rejection is a plain-text
/// body — which would be the one 400 in this API that is not the usual
/// `{ "success": false, "error": { … } }` shape. One line per handler buys that
/// consistency back.
fn parse_id(raw: &str) -> Result<Uuid, AppError> {
    Uuid::parse_str(raw).map_err(|_| AppError::BadRequest("id must be a UUID".into()))
}

// ── Schedule handlers ───────────────────────────────────────────────

/// POST /v1/schedules — create a new schedule and start running it.
#[utoipa::path(
    post,
    path = "/v1/schedules",
    tag = "schedules",
    request_body = CreateScheduleRequest,
    responses(
        (status = 200, description = "Schedule created", body = ApiResponse<Schedule>),
        (status = 400, description = "Invalid action or cron expression"),
    )
)]
pub async fn create_schedule(
    State(pool): State<Pool>,
    State(runner): State<Arc<ScheduleRunner>>,
    Json(body): Json<CreateScheduleRequest>,
) -> Result<Json<ApiResponse<Schedule>>, AppError> {
    Action::parse(&body.action)?;
    ScheduleRunner::validate_cron(&body.cron_expr)?;

    let schedule =
        repo::create_schedule(&pool, body.name, body.cron_expr, body.action).await?;
    runner.sync(&schedule).await?;

    Ok(Json(ApiResponse::new(schedule)))
}

/// GET /v1/schedules — list all schedules.
#[utoipa::path(
    get,
    path = "/v1/schedules",
    tag = "schedules",
    responses(
        (status = 200, description = "List of schedules", body = ApiResponse<Vec<Schedule>>),
        (status = 500, description = "Internal error"),
    )
)]
pub async fn list_schedules(
    State(pool): State<Pool>,
) -> Result<Json<ApiResponse<Vec<Schedule>>>, AppError> {
    Ok(Json(ApiResponse::new(repo::list_schedules(&pool).await?)))
}

/// GET /v1/schedules/{id} — fetch a single schedule.
#[utoipa::path(
    get,
    path = "/v1/schedules/{id}",
    tag = "schedules",
    params(
        ("id" = String, Path, description = "Schedule id (UUID)"),
    ),
    responses(
        (status = 200, description = "The requested schedule", body = ApiResponse<Schedule>),
        (status = 404, description = "Schedule not found"),
    )
)]
pub async fn get_schedule(
    State(pool): State<Pool>,
    Path(id): Path<String>,
) -> Result<Json<ApiResponse<Schedule>>, AppError> {
    let id = parse_id(&id)?;
    Ok(Json(ApiResponse::new(repo::get_schedule(&pool, id).await?)))
}

/// PUT /v1/schedules/{id} — update any subset of a schedule's fields.
///
/// Also re-installs the cron job, so a changed `cron_expr`, `action` or
/// `enabled` flag takes effect immediately rather than at the next restart.
#[utoipa::path(
    put,
    path = "/v1/schedules/{id}",
    tag = "schedules",
    params(
        ("id" = String, Path, description = "Schedule id (UUID)"),
    ),
    request_body = UpdateScheduleRequest,
    responses(
        (status = 200, description = "Schedule updated", body = ApiResponse<Schedule>),
        (status = 400, description = "Invalid action or cron expression"),
        (status = 404, description = "Schedule not found"),
    )
)]
pub async fn update_schedule(
    State(pool): State<Pool>,
    State(runner): State<Arc<ScheduleRunner>>,
    Path(id): Path<String>,
    Json(body): Json<UpdateScheduleRequest>,
) -> Result<Json<ApiResponse<Schedule>>, AppError> {
    let id = parse_id(&id)?;

    if let Some(ref action) = body.action {
        Action::parse(action)?;
    }
    if let Some(ref cron_expr) = body.cron_expr {
        ScheduleRunner::validate_cron(cron_expr)?;
    }

    let schedule = repo::update_schedule(
        &pool,
        id,
        body.name,
        body.cron_expr,
        body.action,
        body.enabled,
    )
    .await?;
    runner.sync(&schedule).await?;

    Ok(Json(ApiResponse::new(schedule)))
}

/// DELETE /v1/schedules/{id} — delete a schedule and stop its job.
#[utoipa::path(
    delete,
    path = "/v1/schedules/{id}",
    tag = "schedules",
    params(
        ("id" = String, Path, description = "Schedule id (UUID)"),
    ),
    responses(
        (status = 200, description = "Schedule deleted", body = ApiResponse<Schedule>),
        (status = 404, description = "Schedule not found"),
    )
)]
pub async fn delete_schedule(
    State(pool): State<Pool>,
    State(runner): State<Arc<ScheduleRunner>>,
    Path(id): Path<String>,
) -> Result<Json<ApiResponse<Schedule>>, AppError> {
    let id = parse_id(&id)?;

    // Delete first: if the row is not there, there is nothing to unschedule and
    // the 404 is the whole answer.
    let schedule = repo::delete_schedule(&pool, id).await?;
    runner.remove(id).await?;

    Ok(Json(ApiResponse::new(schedule)))
}
