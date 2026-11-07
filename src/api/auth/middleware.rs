use axum::{
    extract::{FromRequestParts, Request},
    http::request::Parts,
    middleware::Next,
    response::Response,
};
use uuid::Uuid;

use crate::{api::auth::jwt, error::AppError};

/// Representation of an authenticated user extracted from the JWT access token.
#[derive(Debug, Clone)]
pub struct AuthUser {
    pub id: Uuid,
    pub username: String,
}

impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        // If the auth middleware has already verified and placed the AuthUser into extensions, use it.
        if let Some(user) = parts.extensions.get::<AuthUser>() {
            return Ok(user.clone());
        }

        // Otherwise extract and verify directly from the request headers/cookies.
        let token = jwt::extract_access_token(&parts.headers)
            .ok_or_else(|| AppError::Unauthorized("missing or invalid authorization cookie".into()))?;

        let claims = jwt::verify_token(&token, jwt::TokenType::Access)?;

        Ok(AuthUser {
            id: claims.sub,
            username: claims.username,
        })
    }
}

/// Middleware that protects routes by enforcing a valid JWT access token in the HttpOnly cookie
/// (or Authorization header). On success, the authenticated user is attached to request extensions.
pub async fn auth_middleware(mut req: Request, next: Next) -> Result<Response, AppError> {
    // Exemption: GET /bulb is polled by unauthenticated IoT devices and must remain open.
    if req.method() == axum::http::Method::GET
        && (req.uri().path() == "/bulb" || req.uri().path() == "/bulb/")
    {
        return Ok(next.run(req).await);
    }

    let token = jwt::extract_access_token(req.headers())
        .ok_or_else(|| AppError::Unauthorized("missing or invalid authorization cookie".into()))?;

    let claims = jwt::verify_token(&token, jwt::TokenType::Access)?;

    let auth_user = AuthUser {
        id: claims.sub,
        username: claims.username,
    };

    req.extensions_mut().insert(auth_user);

    Ok(next.run(req).await)
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        extract::Extension,
        http::{header, Request, StatusCode},
        middleware::from_fn,
        routing::get,
        Router,
    };
    use tower::ServiceExt;

    async fn protected_handler(Extension(user): Extension<AuthUser>) -> String {
        format!("hello {}", user.username)
    }

    fn test_app() -> Router {
        Router::new()
            .route("/test", get(protected_handler))
            .layer(from_fn(auth_middleware))
    }

    #[tokio::test]
    async fn test_middleware_rejects_missing_cookie() {
        let app = test_app();
        let req = Request::builder()
            .uri("/test")
            .body(Body::empty())
            .unwrap();

        let res = app.oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_middleware_accepts_valid_access_token_cookie() {
        let user_id = Uuid::new_v4();
        let token = jwt::generate_token(user_id, "bob", jwt::TokenType::Access, 300).unwrap();

        let app = test_app();
        let req = Request::builder()
            .uri("/test")
            .header(header::COOKIE, format!("access_token={token}"))
            .body(Body::empty())
            .unwrap();

        let res = app.oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_middleware_rejects_refresh_token_on_access_endpoint() {
        let user_id = Uuid::new_v4();
        let refresh_token = jwt::generate_token(user_id, "bob", jwt::TokenType::Refresh, 300).unwrap();

        let app = test_app();
        let req = Request::builder()
            .uri("/test")
            .header(header::COOKIE, format!("access_token={refresh_token}"))
            .body(Body::empty())
            .unwrap();

        let res = app.oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_middleware_accepts_bearer_header_fallback() {
        let user_id = Uuid::new_v4();
        let token = jwt::generate_token(user_id, "carol", jwt::TokenType::Access, 300).unwrap();

        let app = test_app();
        let req = Request::builder()
            .uri("/test")
            .header(header::AUTHORIZATION, format!("Bearer {token}"))
            .body(Body::empty())
            .unwrap();

        let res = app.oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_middleware_allows_unauthenticated_get_bulb() {
        let app = Router::new()
            .route("/bulb", get(|| async { "legacy bulb" }))
            .layer(from_fn(auth_middleware));

        let req = Request::builder()
            .uri("/bulb")
            .body(Body::empty())
            .unwrap();

        let res = app.oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::OK);
    }
}


