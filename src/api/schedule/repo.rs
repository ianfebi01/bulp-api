use deadpool_postgres::Pool;
use uuid::Uuid;

use super::dto::Schedule;
use crate::error::AppError;

/// Columns for reading a schedule row, with the timestamps cast to TEXT to
/// match the DTO's `String` fields (raw Postgres rendering, like the bulb v1
/// shape).
const COLS: &str = "id, name, cron_expr, action, enabled, \
                    created_at::TEXT AS created_at, updated_at::TEXT AS updated_at";

fn schedule_from_row(row: &tokio_postgres::Row) -> Schedule {
    Schedule {
        id: row.get("id"),
        name: row.get("name"),
        cron_expr: row.get("cron_expr"),
        action: row.get("action"),
        enabled: row.get("enabled"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

/// Create a schedule. The id comes from the column's `gen_random_uuid()`
/// default, so there is only one place that decides what an id looks like.
pub async fn create_schedule(
    pool: &Pool,
    name: String,
    cron_expr: String,
    action: String,
) -> Result<Schedule, AppError> {
    let sql = format!(
        "INSERT INTO schedules (name, cron_expr, action) \
         VALUES ($1, $2, $3) RETURNING {COLS}"
    );
    let row = pool
        .get()
        .await?
        .query_opt(&sql, &[&name, &cron_expr, &action])
        .await?
        .ok_or_else(|| AppError::Internal("create_schedule returned no row".into()))?;

    Ok(schedule_from_row(&row))
}

/// List all schedules, oldest first.
pub async fn list_schedules(pool: &Pool) -> Result<Vec<Schedule>, AppError> {
    let sql = format!("SELECT {COLS} FROM schedules ORDER BY created_at");
    let rows = pool.get().await?.query(&sql, &[]).await?;
    Ok(rows.iter().map(schedule_from_row).collect())
}

/// Only the schedules that should currently have a cron job — used at startup
/// to rebuild the scheduler from the database.
pub async fn list_enabled(pool: &Pool) -> Result<Vec<Schedule>, AppError> {
    let sql = format!("SELECT {COLS} FROM schedules WHERE enabled ORDER BY created_at");
    let rows = pool.get().await?.query(&sql, &[]).await?;
    Ok(rows.iter().map(schedule_from_row).collect())
}

/// Fetch a single schedule by id.
pub async fn get_schedule(pool: &Pool, id: Uuid) -> Result<Schedule, AppError> {
    let sql = format!("SELECT {COLS} FROM schedules WHERE id = $1");
    let row = pool
        .get()
        .await?
        .query_opt(&sql, &[&id])
        .await?
        .ok_or_else(|| AppError::NotFound("Schedule not found".into()))?;

    Ok(schedule_from_row(&row))
}

/// Update any subset of fields; unspecified fields keep their current value.
pub async fn update_schedule(
    pool: &Pool,
    id: Uuid,
    name: Option<String>,
    cron_expr: Option<String>,
    action: Option<String>,
    enabled: Option<bool>,
) -> Result<Schedule, AppError> {
    let sql = format!(
        "UPDATE schedules SET \
            name = COALESCE($2, name), \
            cron_expr = COALESCE($3, cron_expr), \
            action = COALESCE($4, action), \
            enabled = COALESCE($5, enabled), \
            updated_at = NOW() \
         WHERE id = $1 RETURNING {COLS}"
    );
    let row = pool
        .get()
        .await?
        .query_opt(&sql, &[&id, &name, &cron_expr, &action, &enabled])
        .await?
        .ok_or_else(|| AppError::NotFound("Schedule not found".into()))?;

    Ok(schedule_from_row(&row))
}

/// Delete a schedule and return the removed row.
pub async fn delete_schedule(pool: &Pool, id: Uuid) -> Result<Schedule, AppError> {
    let sql = format!("DELETE FROM schedules WHERE id = $1 RETURNING {COLS}");
    let row = pool
        .get()
        .await?
        .query_opt(&sql, &[&id])
        .await?
        .ok_or_else(|| AppError::NotFound("Schedule not found".into()))?;

    Ok(schedule_from_row(&row))
}
