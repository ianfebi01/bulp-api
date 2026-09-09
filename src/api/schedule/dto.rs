use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::error::AppError;

/// What a schedule does when it fires.
///
/// Stored in Postgres as the TEXT values `"on"` / `"off"` (the column has a
/// CHECK constraint), so the string form stays in [`Schedule`] and this enum is
/// the one place that knows how to read it. Both the HTTP validation and the
/// cron runner go through [`Action::parse`], so they can never disagree about
/// what a valid action is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    On,
    Off,
}

impl Action {
    pub fn parse(action: &str) -> Result<Self, AppError> {
        match action {
            "on" => Ok(Action::On),
            "off" => Ok(Action::Off),
            _ => Err(AppError::BadRequest(
                "action must be \"on\" or \"off\"".into(),
            )),
        }
    }

    /// The bulb state this action writes.
    pub fn is_on(self) -> bool {
        matches!(self, Action::On)
    }
}

/// Response for schedule endpoints.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Schedule {
    /// Model the schema as a string (UUID serializes as a string).
    #[schema(value_type = String)]
    pub id: Uuid,
    pub name: String,
    /// Six space-separated cron fields, **seconds first**, read in the app
    /// timezone (`SCHEDULE_TZ`, default `Asia/Jakarta`) — so `0 53 13 * * *`
    /// fires at 13:53 local time, every day.
    #[schema(example = "0 53 13 * * *")]
    pub cron_expr: String,
    /// "on" or "off"
    pub action: String,
    pub enabled: bool,
    pub created_at: String,
    pub updated_at: String,
}

/// Request body for POST /schedules
#[derive(Deserialize, ToSchema)]
pub struct CreateScheduleRequest {
    pub name: String,
    /// Six cron fields, seconds first, in the app timezone
    /// (`SCHEDULE_TZ`, default `Asia/Jakarta`). Example: 13:53 daily.
    #[schema(example = "0 53 13 * * *")]
    pub cron_expr: String,
    /// Must be "on" or "off"
    pub action: String,
}

/// Request body for PUT /schedules/:id — all fields optional.
#[derive(Deserialize, ToSchema)]
pub struct UpdateScheduleRequest {
    pub name: Option<String>,
    pub cron_expr: Option<String>,
    pub action: Option<String>,
    pub enabled: Option<bool>,
}
