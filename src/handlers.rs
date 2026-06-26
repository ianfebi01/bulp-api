use axum::{extract::State, http::StatusCode, Json};
use std::sync::Arc;

use crate::db::Db;
use crate::models::{BulbState, SetBulbRequest};

pub type AppState = Arc<Db>;

/// GET /bulb — return current bulb state.
pub async fn get_bulb(State(db): State<AppState>) -> Result<Json<BulbState>, StatusCode> {
    db.get_state()
        .map(|(is_on, updated_at)| Json(BulbState { is_on, updated_at }))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

/// POST /bulb/on — turn the bulb on.
pub async fn bulb_on(State(db): State<AppState>) -> Result<Json<BulbState>, StatusCode> {
    db.set_state(true)
        .map(|(is_on, updated_at)| Json(BulbState { is_on, updated_at }))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

/// POST /bulb/off — turn the bulb off.
pub async fn bulb_off(State(db): State<AppState>) -> Result<Json<BulbState>, StatusCode> {
    db.set_state(false)
        .map(|(is_on, updated_at)| Json(BulbState { is_on, updated_at }))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

/// PUT /bulb — set bulb state via JSON body { "is_on": true/false }.
pub async fn set_bulb(
    State(db): State<AppState>,
    Json(body): Json<SetBulbRequest>,
) -> Result<Json<BulbState>, StatusCode> {
    db.set_state(body.is_on)
        .map(|(is_on, updated_at)| Json(BulbState { is_on, updated_at }))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}
