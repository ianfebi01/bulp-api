use utoipa::{
    openapi::security::{ApiKey, ApiKeyValue, SecurityScheme},
    Modify, OpenApi,
};

struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "cookie_auth",
                SecurityScheme::ApiKey(ApiKey::Cookie(ApiKeyValue::new("access_token"))),
            );
        }
    }
}

/// Root OpenAPI document: metadata and security definition.
///
/// Paths and component schemas are collected automatically from the handlers
/// registered through [`utoipa_axum::routes!`] in each resource module's `router()`.
#[derive(OpenApi)]
#[openapi(
    info(
        title = "Bulb API",
        version = "0.1.0",
        description = "IoT bulb control API with JWT authentication"
    ),
    tags(
        (name = "auth", description = "Authentication and user endpoints"),
        (name = "bulb", description = "Bulb state endpoints"),
        (name = "schedules", description = "Cron schedules for the bulb"),
    ),
    modifiers(&SecurityAddon),
    security(
        ("cookie_auth" = [])
    )
)]
pub struct ApiDoc;
