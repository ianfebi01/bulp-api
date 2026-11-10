use argon2::{password_hash::PasswordHasher, Argon2, PasswordHash, PasswordVerifier};
use deadpool_postgres::Pool;

use crate::{
    api::auth::dto::{Login, Register},
    error::AppError,
};

fn is_allowed_user(username: &str) -> bool {
    let allowed_users = std::env::var("ALLOWED_USERS").unwrap_or_default();
    return allowed_users
        .split(',')
        .map(str::trim)
        .any(|u| !u.is_empty() && u == username);
}

const COLS: &str =
    "id, username, password_hash, created_at::TEXT as created_at, updated_at::TEXT as updated_at";

pub async fn register_user(
    pool: &Pool,
    username: String,
    password: String,
) -> Result<Register, AppError> {

    if !is_allowed_user(&username) {
        return Err(AppError::BadRequest("Username not allowed".into()));
    }

    let argon2 = Argon2::default();

    let password_hash = argon2.hash_password(password.as_bytes())?.to_string();

    let sql = format!(
        "INSERT INTO users (username, password_hash) \
        VALUES ($1, $2) RETURNING {COLS}"
    );

    let row = pool
        .get()
        .await?
        .query_opt(&sql, &[&username, &password_hash])
        .await?
        .ok_or_else(|| AppError::Internal("register_user returned no row".into()))?;

    Ok(Register {
        id: row.get("id"),
        username: row.get("username"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    })
}

pub async fn login(pool: &Pool, username: String, password: String) -> Result<Login, AppError> {

    if !is_allowed_user(&username) {
        return Err(AppError::BadRequest("Username not allowed".into()));
    }

    let sql = format!("SELECT {COLS} FROM users WHERE username = $1");

    let row = pool
        .get()
        .await?
        .query_opt(&sql, &[&username])
        .await?
        .ok_or_else(|| AppError::BadRequest("invalid username or password".into()))?;

    let hash_str: &str = row.get("password_hash");
    let parsed_hash = PasswordHash::new(hash_str)
        .map_err(|e| AppError::Internal(format!("invalid password hash in db: {e}")))?;

    Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .map_err(|_| AppError::BadRequest("invalid username or password".into()))?;

    Ok(Login {
        id: row.get("id"),
        username: row.get("username"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    })
}

pub async fn find_user_by_id(pool: &Pool, id: uuid::Uuid) -> Result<Login, AppError> {
    let sql = format!("SELECT {COLS} FROM users WHERE id = $1");

    let row = pool
        .get()
        .await?
        .query_opt(&sql, &[&id])
        .await?
        .ok_or_else(|| AppError::Unauthorized("user not found".into()))?;

    Ok(Login {
        id: row.get("id"),
        username: row.get("username"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    })
}
