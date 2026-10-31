use argon2::{
    Argon2, PasswordHash, PasswordVerifier, password_hash::PasswordHasher,
};
use deadpool_postgres::Pool;

use crate::{api::auth::dto::{Login, Register}, error::AppError};

const COLS: &str = "id, username, password_hash, created_at::TEXT as created_at, updated_at::TEXT as updated_at";

pub async fn register_user(
    pool: &Pool,
    username: String,
    password: String,
) -> Result<Register, AppError> {

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

pub async fn login(
    pool: &Pool,
    username: String,
    password: String,
) -> Result<Login, AppError> {
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
