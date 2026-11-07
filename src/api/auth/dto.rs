use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;


/// Response for register endpoint.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Register {
    /// Model the schema as a string (UUID serializes as a string).
    #[schema(value_type = String)]
    pub id: Uuid,
    pub username: String,
    pub created_at: String,
    pub updated_at: String,
}

/// Request body for POST /register
#[derive(Deserialize, ToSchema)]
pub struct RegisterUserRequest {
    pub username: String,
    pub password: String,
}

/// Response for register endpoint.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Login {
    /// Model the schema as a string (UUID serializes as a string).
    #[schema(value_type = String)]
    pub id: Uuid,
    pub username: String,
    pub created_at: String,
    pub updated_at: String,
}

/// Request body for POST /login
#[derive(Deserialize, ToSchema)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

/// Response for logout endpoint.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct LogoutResponse {
    pub message: String,
}
