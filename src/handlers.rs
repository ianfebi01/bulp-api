use axum::{extract::State, Json};

use crate::error::AppError;
use crate::models::{ApiResponse, BulbState, BulbStateV1, SetBulbRequest};

use deadpool_postgres::Pool;

/// The bulb is a singleton row keyed by `id = 1` (see V1__init.sql).
const BULB_ID: i32 = 1;

/// Map a `bulb_state` row into the v2 response model.
fn bulb_state_from_row(row: &tokio_postgres::Row) -> BulbState {
    BulbState {
        is_on: row.get("is_on"),
        updated_at: row.get("updated_at"),
    }
}

// ── Bulb handlers ───────────────────────────────────────────────────

/// GET /bulb — v1, legacy flat response consumed by the IoT device.
///
/// Keeps the original shape `{ is_on, updated_at }` with `updated_at` as the
/// raw Postgres text rendering. Do NOT change this — devices depend on it.
/// New clients should use `GET /v2/bulb` (see [`get_bulb_v2`]).
pub async fn get_bulb(State(pool): State<Pool>) -> Result<Json<BulbStateV1>, AppError> {
    let client = pool.get().await?;

    let row = client
        .query_opt(
            "SELECT is_on, updated_at::TEXT FROM bulb_state WHERE id = $1",
            &[&BULB_ID],
        )
        .await?
        .ok_or_else(|| AppError::NotFound("Bulb state not found".into()))?;

    Ok(Json(BulbStateV1 {
        is_on: row.get("is_on"),
        updated_at: row.get("updated_at"),
    }))
}

/// GET /v2/bulb — return current bulb state in the standard envelope.
pub async fn get_bulb_v2(
    State(pool): State<Pool>,
) -> Result<Json<ApiResponse<BulbState>>, AppError> {
    let client = pool.get().await?;

    let row = client
        .query_opt(
            "SELECT is_on, updated_at FROM bulb_state WHERE id = $1",
            &[&BULB_ID],
        )
        .await?
        .ok_or_else(|| AppError::NotFound("Bulb state not found".into()))?;

    Ok(Json(ApiResponse::new(bulb_state_from_row(&row))))
}

/// POST /bulb/on — turn the bulb on.
pub async fn bulb_on(State(pool): State<Pool>) -> Result<Json<ApiResponse<BulbState>>, AppError> {
    let client = pool.get().await?;

    let row = client
        .query_opt(
            "UPDATE bulb_state SET is_on = TRUE, updated_at = NOW() \
                        WHERE id = $1 RETURNING is_on, updated_at",
            &[&BULB_ID],
        )
        .await?
        .ok_or_else(|| AppError::NotFound("Bulb state not found".into()))?;

    Ok(Json(ApiResponse::new(bulb_state_from_row(&row))))
}

/// POST /bulb/off — turn the bulb off.
pub async fn bulb_off(State(pool): State<Pool>) -> Result<Json<ApiResponse<BulbState>>, AppError> {
    let client = pool.get().await?;

    let row = client
        .query_opt(
            "UPDATE bulb_state SET is_on = FALSE, updated_at = NOW() \
                        WHERE id = $1 RETURNING is_on, updated_at",
            &[&BULB_ID],
        )
        .await?
        .ok_or_else(|| AppError::NotFound("Bulb state not found".into()))?;

    Ok(Json(ApiResponse::new(bulb_state_from_row(&row))))
}

/// PUT /bulb — set bulb state via JSON body { "is_on": true/false }.

pub async fn set_bulb(
    State(pool): State<Pool>,
    Json(body): Json<SetBulbRequest>,
) -> Result<Json<ApiResponse<BulbState>>, AppError> {

    let client = pool.get().await?;

    let is_on = body.is_on;

    let row = client
    .query_opt("UPDATE bulb_state SET is_on = $2, updated_at = NOW() \
                            WHERE id = $1 \
                            RETURNING is_on, updated_at", &[&1_i32, &is_on])
    .await?
    .ok_or_else(|| AppError::NotFound("Bulb state not found".into()))?;

    Ok(Json(ApiResponse::new(bulb_state_from_row(&row))))
}

// // ── Schedule handlers ───────────────────────────────────────────────

// /// POST /schedules — create a new schedule and register its cron job.
// pub async fn create_schedule(
//     State(state): State<AppState>,
//     Json(body): Json<CreateScheduleRequest>,
// ) -> Result<(StatusCode, Json<Schedule>), StatusCode> {
//     // Validate action
//     if body.action != "on" && body.action != "off" {
//         return Err(StatusCode::BAD_REQUEST);
//     }

//     let schedule = state
//         .db
//         .create_schedule(&body.name, &body.cron_expr, &body.action)
//         .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

//     // Register the cron job
//     state
//         .scheduler
//         .add_job(&schedule)
//         .await
//         .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

//     Ok((StatusCode::CREATED, Json(schedule)))
// }

// /// GET /schedules — list all schedules.
// pub async fn list_schedules(
//     State(state): State<AppState>,
// ) -> Result<Json<Vec<Schedule>>, StatusCode> {
//     state
//         .db
//         .list_schedules()
//         .map(Json)
//         .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
// }

// /// GET /schedules/:id — get a single schedule.
// pub async fn get_schedule(
//     State(state): State<AppState>,
//     Path(id): Path<String>,
// ) -> Result<Json<Schedule>, StatusCode> {
//     state
//         .db
//         .get_schedule(&id)
//         .map(Json)
//         .map_err(|_| StatusCode::NOT_FOUND)
// }

// /// PUT /schedules/:id — update a schedule and reload its cron job.
// pub async fn update_schedule(
//     State(state): State<AppState>,
//     Path(id): Path<String>,
//     Json(body): Json<UpdateScheduleRequest>,
// ) -> Result<Json<Schedule>, StatusCode> {
//     // Validate action if provided
//     if let Some(ref action) = body.action {
//         if action != "on" && action != "off" {
//             return Err(StatusCode::BAD_REQUEST);
//         }
//     }

//     let schedule = state
//         .db
//         .update_schedule(
//             &id,
//             body.name.as_deref(),
//             body.cron_expr.as_deref(),
//             body.action.as_deref(),
//             body.enabled,
//         )
//         .map_err(|_| StatusCode::NOT_FOUND)?;

//     // Reload the cron job (removes old, adds new if enabled)
//     state
//         .scheduler
//         .reload_job(&schedule)
//         .await
//         .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

//     Ok(Json(schedule))
// }

// /// DELETE /schedules/:id — delete a schedule and remove its cron job.
// pub async fn delete_schedule(
//     State(state): State<AppState>,
//     Path(id): Path<String>,
// ) -> Result<StatusCode, StatusCode> {
//     // Remove cron job first
//     state
//         .scheduler
//         .remove_job(&id)
//         .await
//         .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

//     let deleted = state
//         .db
//         .delete_schedule(&id)
//         .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

//     if deleted {
//         Ok(StatusCode::NO_CONTENT)
//     } else {
//         Err(StatusCode::NOT_FOUND)
//     }
// }

use axum::response::IntoResponse;

pub async fn not_found() -> impl IntoResponse {
    AppError::NotFound("route not found".into())
}
