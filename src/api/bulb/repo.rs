//! All SQL for the `bulb_state` table. Handlers stay free of query strings.

use deadpool_postgres::Pool;

use super::dto::{BulbState, BulbStateV1};
use crate::error::AppError;

/// The bulb is a singleton row keyed by `id = 1` (see V1__init.sql).
const BULB_ID: i32 = 1;

fn state_from_row(row: &tokio_postgres::Row) -> BulbState {
    BulbState {
        is_on: row.get("is_on"),
        updated_at: row.get("updated_at"),
    }
}

fn missing() -> AppError {
    AppError::NotFound("Bulb state not found".into())
}

/// Current state with `updated_at` rendered as Postgres text (v1 shape).
pub async fn find_v1(pool: &Pool) -> Result<BulbStateV1, AppError> {
    let row = pool
        .get()
        .await?
        .query_opt(
            "SELECT is_on, updated_at::TEXT FROM bulb_state WHERE id = $1",
            &[&BULB_ID],
        )
        .await?
        .ok_or_else(missing)?;

    Ok(BulbStateV1 {
        is_on: row.get("is_on"),
        updated_at: row.get("updated_at"),
    })
}

/// Current state with a typed timestamp.
pub async fn find(pool: &Pool) -> Result<BulbState, AppError> {
    let row = pool
        .get()
        .await?
        .query_opt(
            "SELECT is_on, updated_at FROM bulb_state WHERE id = $1",
            &[&BULB_ID],
        )
        .await?
        .ok_or_else(missing)?;

    Ok(state_from_row(&row))
}

/// Flip the bulb on or off and return the state that was written.
pub async fn set(pool: &Pool, is_on: bool) -> Result<BulbState, AppError> {
    let row = pool
        .get()
        .await?
        .query_opt(
            "UPDATE bulb_state SET is_on = $2, updated_at = NOW() \
             WHERE id = $1 \
             RETURNING is_on, updated_at",
            &[&BULB_ID, &is_on],
        )
        .await?
        .ok_or_else(missing)?;

    Ok(state_from_row(&row))
}
