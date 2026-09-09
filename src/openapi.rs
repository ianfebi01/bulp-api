use utoipa::OpenApi;

/// Root OpenAPI document: metadata only.
///
/// Paths and component schemas are **not** listed here — they are collected
/// automatically from the handlers registered through
/// [`utoipa_axum::routes!`] in each resource module's `router()`. That is what
/// keeps the served routes and the documented routes from ever drifting apart.
#[derive(OpenApi)]
#[openapi(
    info(
        title = "Bulb API",
        version = "0.1.0",
        description = "IoT bulb control API"
    ),
    tags(
        (name = "bulb", description = "Bulb state endpoints"),
        (name = "schedules", description = "Cron schedules for the bulb"),
    )
)]
pub struct ApiDoc;
