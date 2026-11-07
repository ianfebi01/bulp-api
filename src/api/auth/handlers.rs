use axum::{
    extract::State,
    http::{header, HeaderMap, StatusCode},
    Json,
};
use deadpool_postgres::Pool;

use crate::{
    api::{
        auth::{
            dto::{Login, LoginRequest, LogoutResponse, Register, RegisterUserRequest},
            jwt,
            middleware::AuthUser,
            repo,
        },
        response::ApiResponse,
    },
    error::AppError,
};

/// POST /v1/auth/register — create a new user and set HTTP-only auth cookies.
#[utoipa::path(
    post,
    path = "/v1/auth/register",
    tag = "auth",
    security(()),
    request_body = RegisterUserRequest,
    responses(
        (status = 201, description = "User registered", body = ApiResponse<Register>),
        (status = 400, description = "Invalid request"),
    )
)]
pub async fn register_user(
    State(pool): State<Pool>,
    Json(body): Json<RegisterUserRequest>,
) -> Result<(StatusCode, HeaderMap, Json<ApiResponse<Register>>), AppError> {
    let register = repo::register_user(&pool, body.username, body.password).await?;
    let (access_token, refresh_token) = jwt::generate_token_pair(register.id, &register.username)?;

    let mut headers = HeaderMap::new();
    let (access_cookie, refresh_cookie) = jwt::build_auth_cookies(&access_token, &refresh_token);
    headers.append(header::SET_COOKIE, access_cookie);
    headers.append(header::SET_COOKIE, refresh_cookie);

    Ok((StatusCode::CREATED, headers, Json(ApiResponse::new(register))))
}

/// POST /v1/auth/login — authenticate a user and set HTTP-only auth cookies.
#[utoipa::path(
    post,
    path = "/v1/auth/login",
    tag = "auth",
    security(()),
    request_body = LoginRequest,
    responses(
        (status = 200, description = "User logged in", body = ApiResponse<Login>),
        (status = 400, description = "Invalid request"),
    )
)]
pub async fn login(
    State(pool): State<Pool>,
    Json(body): Json<LoginRequest>,
) -> Result<(HeaderMap, Json<ApiResponse<Login>>), AppError> {
    let login = repo::login(&pool, body.username, body.password).await?;
    let (access_token, refresh_token) = jwt::generate_token_pair(login.id, &login.username)?;

    let mut headers = HeaderMap::new();
    let (access_cookie, refresh_cookie) = jwt::build_auth_cookies(&access_token, &refresh_token);
    headers.append(header::SET_COOKIE, access_cookie);
    headers.append(header::SET_COOKIE, refresh_cookie);

    Ok((headers, Json(ApiResponse::new(login))))
}

/// POST /v1/auth/refresh — refresh access and refresh tokens using the HTTP-only refresh_token cookie.
#[utoipa::path(
    post,
    path = "/v1/auth/refresh",
    tag = "auth",
    security(()),
    responses(
        (status = 200, description = "Tokens refreshed", body = ApiResponse<Login>),
        (status = 401, description = "Missing or invalid refresh token"),
    )
)]
pub async fn refresh(
    State(pool): State<Pool>,
    headers: HeaderMap,
) -> Result<(HeaderMap, Json<ApiResponse<Login>>), AppError> {
    let refresh_token = jwt::extract_cookie(&headers, jwt::REFRESH_COOKIE_NAME)
        .ok_or_else(|| AppError::Unauthorized("missing refresh token cookie".into()))?;

    let claims = jwt::verify_token(&refresh_token, jwt::TokenType::Refresh)?;
    let user = repo::find_user_by_id(&pool, claims.sub).await?;

    let (new_access_token, new_refresh_token) = jwt::generate_token_pair(user.id, &user.username)?;

    let mut res_headers = HeaderMap::new();
    let (access_cookie, refresh_cookie) =
        jwt::build_auth_cookies(&new_access_token, &new_refresh_token);
    res_headers.append(header::SET_COOKIE, access_cookie);
    res_headers.append(header::SET_COOKIE, refresh_cookie);

    Ok((res_headers, Json(ApiResponse::new(user))))
}

/// POST /v1/auth/logout — clear HTTP-only auth cookies.
#[utoipa::path(
    post,
    path = "/v1/auth/logout",
    tag = "auth",
    security(()),
    responses(
        (status = 200, description = "Logged out successfully", body = ApiResponse<LogoutResponse>),
    )
)]
pub async fn logout() -> Result<(HeaderMap, Json<ApiResponse<LogoutResponse>>), AppError> {
    let mut headers = HeaderMap::new();
    headers.append(header::SET_COOKIE, jwt::clear_cookie(jwt::ACCESS_COOKIE_NAME));
    headers.append(header::SET_COOKIE, jwt::clear_cookie(jwt::REFRESH_COOKIE_NAME));

    Ok((
        headers,
        Json(ApiResponse::new(LogoutResponse {
            message: "logged out successfully".into(),
        })),
    ))
}

/// GET /v1/auth/me — retrieve the currently authenticated user's profile.
#[utoipa::path(
    get,
    path = "/v1/auth/me",
    tag = "auth",
    responses(
        (status = 200, description = "Current authenticated user", body = ApiResponse<Login>),
        (status = 401, description = "Unauthorized"),
    )
)]
pub async fn me(
    State(pool): State<Pool>,
    auth_user: AuthUser,
) -> Result<Json<ApiResponse<Login>>, AppError> {
    let user = repo::find_user_by_id(&pool, auth_user.id).await?;
    Ok(Json(ApiResponse::new(user)))
}
