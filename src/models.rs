use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Standard success envelope for every handler:
/// `{ "success": true, "data": ... }`.
#[derive(Serialize)]
pub struct ApiResponse<T: Serialize> {
    pub success: bool,
    pub data: T,
}

impl<T: Serialize> ApiResponse<T> {
    pub fn new(data: T) -> Self {
        Self { success: true, data }
    }
}

/// Legacy (v1) bulb response — flat shape consumed by the IoT device.
/// `updated_at` is the raw Postgres text rendering (e.g. `2026-07-27 10:40:00+00`).
/// Do NOT change this shape; existing devices depend on it.
#[derive(Serialize)]
pub struct BulbStateV1 {
    pub is_on: bool,
    pub updated_at: String,
}

/// v2 bulb response — typed timestamp, returned inside `ApiResponse`.
#[derive(Serialize)]
pub struct BulbState {
    pub is_on: bool,
    /// Serialized as RFC 3339 (e.g. `2026-07-27T10:40:00Z`).
    pub updated_at: DateTime<Utc>,
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
