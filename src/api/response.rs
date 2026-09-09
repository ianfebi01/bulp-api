use serde::Serialize;
use utoipa::ToSchema;

/// Standard success envelope for every handler:
/// `{ "success": true, "data": ... }`.
#[derive(Serialize, ToSchema)]
pub struct ApiResponse<T: Serialize> {
    pub success: bool,
    pub data: T,
}

impl<T: Serialize> ApiResponse<T> {
    pub fn new(data: T) -> Self {
        Self { success: true, data }
    }
}
