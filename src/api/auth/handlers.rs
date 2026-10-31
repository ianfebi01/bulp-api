use axum::{Json, extract::State};
use deadpool_postgres::Pool;

use crate::{api::{auth::dto::{Login, LoginRequest, Register, RegisterUserRequest}, response::ApiResponse}, error::AppError};


/// POST /v1/auth/register — create a new user.
#[utoipa::path(
    post,
    path = "/v1/auth/register",
    tag = "auth",
    request_body = RegisterUserRequest,
    responses(
        (status = 201, description = "User registered", body = ApiResponse<Register>),
        (status = 400, description = "Invalid request"),
    )
)]

pub async fn register_user(
    State(pool): State<Pool>,
    Json(body): Json<RegisterUserRequest>,
) -> Result<Json<ApiResponse<Register>>, AppError> {
    
    let register = crate::api::auth::repo::register_user(&pool, body.username, body.password).await?;
    Ok(Json(ApiResponse::new(register)))
}

/// POST /v1/auth/login — authenticate a user.
#[utoipa::path(
    post,
    path = "/v1/auth/login",
    tag = "auth",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "User logged in", body = ApiResponse<Login>),
        (status = 400, description = "Invalid request"),
    )
)]

pub async fn login(
    State(pool): State<Pool>,
    Json(body): Json<LoginRequest>,
) -> Result<Json<ApiResponse<Login>>, AppError> {
    
    let login = crate::api::auth::repo::login(&pool, body.username, body.password).await?;
    Ok(Json(ApiResponse::new(login)))
}