use std::env;

use axum::http::{header, HeaderMap, HeaderValue};
use chrono::Utc;
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::AppError;

pub const ACCESS_COOKIE_NAME: &str = "access_token";
pub const REFRESH_COOKIE_NAME: &str = "refresh_token";

const DEFAULT_JWT_SECRET: &str = "bulb-api-jwt-secret-key-change-in-production";
const DEFAULT_ACCESS_EXPIRY_SECS: i64 = 900; // 15 minutes
const DEFAULT_REFRESH_EXPIRY_SECS: i64 = 604800; // 7 days

pub fn jwt_secret() -> String {
    env::var("JWT_SECRET").unwrap_or_else(|_| DEFAULT_JWT_SECRET.to_string())
}

pub fn access_token_max_age_secs() -> i64 {
    env::var("JWT_ACCESS_EXPIRY_SECS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(DEFAULT_ACCESS_EXPIRY_SECS)
}

pub fn refresh_token_max_age_secs() -> i64 {
    env::var("JWT_REFRESH_EXPIRY_SECS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(DEFAULT_REFRESH_EXPIRY_SECS)
}

pub fn is_cookie_secure() -> bool {
    env::var("COOKIE_SECURE")
        .map(|v| v == "true" || v == "1")
        .unwrap_or(false)
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TokenType {
    Access,
    Refresh,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: Uuid,
    pub username: String,
    pub exp: i64,
    pub iat: i64,
    pub token_type: TokenType,
}

/// Generate a signed JWT token with the given type and duration.
pub fn generate_token(
    user_id: Uuid,
    username: &str,
    token_type: TokenType,
    expiry_secs: i64,
) -> Result<String, AppError> {
    let now = Utc::now().timestamp();
    let claims = Claims {
        sub: user_id,
        username: username.to_string(),
        exp: now + expiry_secs,
        iat: now,
        token_type,
    };

    let secret = jwt_secret();
    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )?;

    Ok(token)
}

/// Convenience to generate both an access token and a refresh token for a user.
pub fn generate_token_pair(user_id: Uuid, username: &str) -> Result<(String, String), AppError> {
    let access_token = generate_token(
        user_id,
        username,
        TokenType::Access,
        access_token_max_age_secs(),
    )?;

    let refresh_token = generate_token(
        user_id,
        username,
        TokenType::Refresh,
        refresh_token_max_age_secs(),
    )?;

    Ok((access_token, refresh_token))
}

/// Verify and decode a JWT token, ensuring it has not expired and matches the expected token type.
pub fn verify_token(token: &str, expected_type: TokenType) -> Result<Claims, AppError> {
    let secret = jwt_secret();
    let mut validation = Validation::new(Algorithm::HS256);
    validation.validate_aud = false;

    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &validation,
    )?;

    if token_data.claims.token_type != expected_type {
        return Err(AppError::Unauthorized(format!(
            "invalid token type, expected {:?}",
            expected_type
        )));
    }

    Ok(token_data.claims)
}

/// Format an HTTP-only Set-Cookie header value with standard security flags.
pub fn make_cookie(name: &str, value: &str, max_age_secs: i64) -> HeaderValue {
    let secure_flag = if is_cookie_secure() { "; Secure" } else { "" };
    let cookie = format!(
        "{name}={value}; HttpOnly; Path=/; SameSite=Lax; Max-Age={max_age_secs}{secure_flag}"
    );
    HeaderValue::from_str(&cookie).expect("valid cookie header")
}

/// Format a Set-Cookie header to immediately expire/clear the cookie.
pub fn clear_cookie(name: &str) -> HeaderValue {
    let secure_flag = if is_cookie_secure() { "; Secure" } else { "" };
    let cookie = format!("{name}=; HttpOnly; Path=/; SameSite=Lax; Max-Age=0{secure_flag}");
    HeaderValue::from_str(&cookie).expect("valid cookie header")
}

/// Build both access and refresh Set-Cookie header values.
pub fn build_auth_cookies(access_token: &str, refresh_token: &str) -> (HeaderValue, HeaderValue) {
    (
        make_cookie(
            ACCESS_COOKIE_NAME,
            access_token,
            access_token_max_age_secs(),
        ),
        make_cookie(
            REFRESH_COOKIE_NAME,
            refresh_token,
            refresh_token_max_age_secs(),
        ),
    )
}

/// Extract a cookie value by name from the HTTP request headers.
pub fn extract_cookie(headers: &HeaderMap, name: &str) -> Option<String> {
    headers
        .get(header::COOKIE)?
        .to_str()
        .ok()?
        .split(';')
        .find_map(|pair| {
            let (key, val) = pair.trim().split_once('=')?;
            if key == name {
                Some(val.to_string())
            } else {
                None
            }
        })
}

/// Extract the access token either from the HttpOnly `access_token` cookie
/// or as a fallback from the `Authorization: Bearer <token>` header.
pub fn extract_access_token(headers: &HeaderMap) -> Option<String> {
    if let Some(token) = extract_cookie(headers, ACCESS_COOKIE_NAME) {
        return Some(token);
    }

    // Fallback: Bearer token in Authorization header
    headers
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|auth| auth.strip_prefix("Bearer ").or_else(|| auth.strip_prefix("bearer ")))
        .map(|s| s.trim().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_generation_and_verification() {
        let user_id = Uuid::new_v4();
        let username = "alice";

        let (access_token, refresh_token) = generate_token_pair(user_id, username).unwrap();

        // Validating access token as access token should succeed
        let access_claims = verify_token(&access_token, TokenType::Access).unwrap();
        assert_eq!(access_claims.sub, user_id);
        assert_eq!(access_claims.username, username);
        assert_eq!(access_claims.token_type, TokenType::Access);

        // Validating refresh token as refresh token should succeed
        let refresh_claims = verify_token(&refresh_token, TokenType::Refresh).unwrap();
        assert_eq!(refresh_claims.sub, user_id);
        assert_eq!(refresh_claims.username, username);
        assert_eq!(refresh_claims.token_type, TokenType::Refresh);

        // Validating refresh token as access token should fail
        let mismatch = verify_token(&refresh_token, TokenType::Access);
        assert!(mismatch.is_err());

        // Validating tampered token should fail
        let tampered = format!("{access_token}tampered");
        assert!(verify_token(&tampered, TokenType::Access).is_err());
    }

    #[test]
    fn test_cookie_generation_and_extraction() {
        let access_val = "test-access-token-123";
        let refresh_val = "test-refresh-token-456";

        let (access_cookie, refresh_cookie) = build_auth_cookies(access_val, refresh_val);
        let access_str = access_cookie.to_str().unwrap();
        assert!(access_str.contains("access_token=test-access-token-123"));
        assert!(access_str.contains("HttpOnly"));
        assert!(access_str.contains("Path=/"));
        assert!(access_str.contains("SameSite=Lax"));

        let refresh_str = refresh_cookie.to_str().unwrap();
        assert!(refresh_str.contains("refresh_token=test-refresh-token-456"));
        assert!(refresh_str.contains("HttpOnly"));

        let clear_cookie = clear_cookie(ACCESS_COOKIE_NAME);
        let clear_str = clear_cookie.to_str().unwrap();
        assert!(clear_str.contains("Max-Age=0"));

        let mut headers = HeaderMap::new();
        headers.append(header::COOKIE, HeaderValue::from_str("other=1; access_token=secret_abc; foo=bar").unwrap());
        let extracted = extract_cookie(&headers, ACCESS_COOKIE_NAME);
        assert_eq!(extracted.as_deref(), Some("secret_abc"));
    }

    #[test]
    fn test_extract_access_token_bearer_fallback() {
        let mut headers = HeaderMap::new();
        headers.insert(header::AUTHORIZATION, HeaderValue::from_static("Bearer my_bearer_token"));

        let token = extract_access_token(&headers);
        assert_eq!(token.as_deref(), Some("my_bearer_token"));
    }
}

