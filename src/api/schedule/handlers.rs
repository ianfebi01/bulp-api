//! Schedule HTTP layer: extract, delegate to [`super::repo`], wrap the response.

use axum::{
    extract::{Path, State},
    Json,
};
use deadpool_postgres::Pool;

use super::dto::{CreateScheduleRequest, Schedule, UpdateScheduleRequest};
use super::repo;
use crate::api::response::ApiResponse;
use crate::error::AppError;

/// Validate the `action` field — must be "on" or "off".
fn check_action(action: &str) -> Result<(), AppError> {
    if action == "on" || action == "off" {
        Ok(())
    } else {
        Err(AppError::BadRequest("action must be \"on\" or \"off\"".into()))
    }
}

// ── Schedule handlers ───────────────────────────────────────────────

/// POST /v1/schedules — create a new schedule.
#[utoipa::path(
    post,
    path = "/v1/schedules",
    tag = "schedules",
    request_body = CreateScheduleRequest,
    responses(
        (status = 200, description = "Schedule created", body = ApiResponse<Schedule>),
        (status = 400, description = "Invalid request body"),
    )
)]
pub async fn create_schedule(
    State(pool): State<Pool>,
    Json(body): Json<CreateScheduleRequest>,
) -> Result<Json<ApiResponse<Schedule>>, AppError> {
    check_action(&body.action)?;
    let schedule =
        repo::create_schedule(&pool, body.name, body.cron_expr, body.action).await?;
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
        ("id" = String, Path, description = "Schedule id"),
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
    Ok(Json(ApiResponse::new(repo::get_schedule(&pool, &id).await?)))
}

/// PUT /v1/schedules/{id} — update any subset of a schedule's fields.
#[utoipa::path(
    put,
    path = "/v1/schedules/{id}",
    tag = "schedules",
    params(
        ("id" = String, Path, description = "Schedule id"),
    ),
    request_body = UpdateScheduleRequest,
    responses(
        (status = 200, description = "Schedule updated", body = ApiResponse<Schedule>),
        (status = 400, description = "Invalid request body"),
        (status = 404, description = "Schedule not found"),
    )
)]
pub async fn update_schedule(
    State(pool): State<Pool>,
    Path(id): Path<String>,
    Json(body): Json<UpdateScheduleRequest>,
) -> Result<Json<ApiResponse<Schedule>>, AppError> {
    if let Some(ref action) = body.action {
        check_action(action)?;
    }
    let schedule = repo::update_schedule(
        &pool,
        &id,
        body.name,
        body.cron_expr,
        body.action,
        body.enabled,
    )
    .await?;
    Ok(Json(ApiResponse::new(schedule)))
}

/// DELETE /v1/schedules/{id} — delete a schedule.
#[utoipa::path(
    delete,
    path = "/v1/schedules/{id}",
    tag = "schedules",
    params(
        ("id" = String, Path, description = "Schedule id"),
    ),
    responses(
        (status = 200, description = "Schedule deleted", body = ApiResponse<Schedule>),
        (status = 404, description = "Schedule not found"),
    )
)]
pub async fn delete_schedule(
    State(pool): State<Pool>,
    Path(id): Path<String>,
) -> Result<Json<ApiResponse<Schedule>>, AppError> {
    Ok(Json(ApiResponse::new(repo::delete_schedule(&pool, &id).await?)))
}
