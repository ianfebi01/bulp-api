use argon2::{
    password_hash::{PasswordHasher},
    Argon2,
};
use deadpool_postgres::Pool;

use crate::{api::auth::dto::Register, error::AppError};

const COLS: &str = "id, username, password_hash, created_at::TEXT as created_at, updated_at::TEXT as updated_at";

pub async fn register_user(
    pool: &Pool,
    username: String,
    password: String,
) -> Result<Register, AppError> {

    let argon2 = Argon2::default();

    let password_hash = argon2.hash_password(password.as_bytes())?.to_string();

    println!("Password hash: {}", password_hash); // Debugging line

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
