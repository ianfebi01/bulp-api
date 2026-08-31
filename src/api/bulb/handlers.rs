//! Thin HTTP layer: extract, delegate to [`super::repo`], wrap the response.
//!
//! Each handler's `#[utoipa::path]` is the single declaration of its URL and
//! method — `super::router()` feeds the very same annotation to axum and to
//! the OpenAPI document.

use axum::{extract::State, Json};
use deadpool_postgres::Pool;

use super::dto::{BulbState, BulbStateV1, SetBulbRequest};
use super::repo;
use crate::api::response::ApiResponse;
use crate::error::AppError;

/// GET /bulb — v1, legacy flat response consumed by the IoT device.
///
/// Keeps the original shape `{ is_on, updated_at }` with `updated_at` as the
/// raw Postgres text rendering. Do NOT change this — devices depend on it.
/// New clients should use `GET /v2/bulb` (see [`get_bulb_v2`]).
#[utoipa::path(
    get,
    path = "/bulb",
    tag = "bulb",
    responses(
        (status = 200, description = "Current bulb state (legacy v1 shape)", body = BulbStateV1),
        (status = 404, description = "Bulb state row not found"),
        (status = 500, description = "Internal error"),
    )
)]
pub async fn get_bulb(State(pool): State<Pool>) -> Result<Json<BulbStateV1>, AppError> {
    Ok(Json(repo::find_v1(&pool).await?))
}

/// GET /v2/bulb — return current bulb state in the standard envelope.
#[utoipa::path(
    get,
    path = "/v2/bulb",
    tag = "bulb",
    responses(
        (status = 200, description = "Current bulb state", body = ApiResponse<BulbState>),
        (status = 404, description = "Bulb state row not found"),
        (status = 500, description = "Internal error"),
    )
)]
pub async fn get_bulb_v2(
    State(pool): State<Pool>,
) -> Result<Json<ApiResponse<BulbState>>, AppError> {
    Ok(Json(ApiResponse::new(repo::find(&pool).await?)))
}

/// PUT /bulb — set bulb state via JSON body { "is_on": true/false }.
#[utoipa::path(
    put,
    path = "/bulb",
    tag = "bulb",
    request_body = SetBulbRequest,
    responses(
        (status = 200, description = "Bulb state updated", body = ApiResponse<BulbState>),
        (status = 404, description = "Bulb state row not found"),
    )
)]
pub async fn set_bulb(
    State(pool): State<Pool>,
    Json(body): Json<SetBulbRequest>,
) -> Result<Json<ApiResponse<BulbState>>, AppError> {
    Ok(Json(ApiResponse::new(repo::set(&pool, body.is_on).await?)))
}

/// POST /bulb/on — turn the bulb on.
#[utoipa::path(
    post,
    path = "/bulb/on",
    tag = "bulb",
    responses(
        (status = 200, description = "Bulb turned on", body = ApiResponse<BulbState>),
        (status = 404, description = "Bulb state row not found"),
    )
)]
pub async fn bulb_on(State(pool): State<Pool>) -> Result<Json<ApiResponse<BulbState>>, AppError> {
    Ok(Json(ApiResponse::new(repo::set(&pool, true).await?)))
}

/// POST /bulb/off — turn the bulb off.
#[utoipa::path(
    post,
    path = "/bulb/off",
    tag = "bulb",
    responses(
        (status = 200, description = "Bulb turned off", body = ApiResponse<BulbState>),
        (status = 404, description = "Bulb state row not found"),
    )
)]
pub async fn bulb_off(State(pool): State<Pool>) -> Result<Json<ApiResponse<BulbState>>, AppError> {
    Ok(Json(ApiResponse::new(repo::set(&pool, false).await?)))
}
