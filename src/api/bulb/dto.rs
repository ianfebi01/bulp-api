use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Legacy (v1) bulb response — flat shape consumed by the IoT device.
/// `updated_at` is the raw Postgres text rendering (e.g. `2026-07-27 10:40:00+00`).
/// Do NOT change this shape; existing devices depend on it.
#[derive(Serialize, ToSchema)]
pub struct BulbStateV1 {
    /// Whether the bulb is currently on.
    #[schema(example = true)]
    pub is_on: bool,
    /// Raw Postgres text rendering of the last change.
    #[schema(example = "2026-07-27 10:40:00+00")]
    pub updated_at: String,
}

/// v2 bulb response — typed timestamp, returned inside `ApiResponse`.
#[derive(Serialize, ToSchema)]
pub struct BulbState {
    #[schema(example = true)]
    pub is_on: bool,
    /// Serialized as RFC 3339 (e.g. `2026-07-27T10:40:00Z`).
    #[schema(example = "2026-07-27T10:40:00Z")]
    pub updated_at: DateTime<Utc>,
}

/// Request body for PUT /bulb
#[derive(Deserialize, ToSchema)]
pub struct SetBulbRequest {
    #[schema(example = true)]
    pub is_on: bool,
}
