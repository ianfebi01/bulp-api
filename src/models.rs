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
