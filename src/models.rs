use serde::{Deserialize, Serialize};

/// Response for GET /bulb and PUT /bulb
#[derive(Serialize)]
pub struct BulbState {
    pub is_on: bool,
    pub updated_at: String,
}

/// Request body for PUT /bulb
#[derive(Deserialize)]
pub struct SetBulbRequest {
    pub is_on: bool,
}

// ── Schedule types ──────────────────────────────────────────────────

/// Response for schedule endpoints.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Schedule {
    pub id: String,
    pub name: String,
    pub cron_expr: String,
    /// "on" or "off"
    pub action: String,
    pub enabled: bool,
    pub created_at: String,
    pub updated_at: String,
}

/// Request body for POST /schedules
#[derive(Deserialize)]
pub struct CreateScheduleRequest {
    pub name: String,
    pub cron_expr: String,
    /// Must be "on" or "off"
    pub action: String,
}

/// Request body for PUT /schedules/:id — all fields optional.
#[derive(Deserialize)]
pub struct UpdateScheduleRequest {
    pub name: Option<String>,
    pub cron_expr: Option<String>,
    pub action: Option<String>,
    pub enabled: Option<bool>,
}
