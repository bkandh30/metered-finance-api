use axum::Router;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

#[derive(OpenApi)]
#[openapi(
    info(
        title = "Metered Finance API",
        version = "1.0.0",
        description = "Production ready Transactions API for Fintech Companies with metering",
        contact(
            name = "API Support",
            email = "support@financely.com",
        ),
        license(
            name = "MIT"
        )
    ),
    servers(
        (url = "http://localhost:3030", description = "Local Development Server"),
        (url = "https://staging.api.financely.com", description = "Staging Server"),
        (url = "https://api.financely.com", description = "Production Server"),
    ),
    paths(
        // Paths here
    ),
    components(
        schemas(
            // Schemas here
        )
    ),
    tags(
        (name = "health", description = "Health Check Endpoints"),
    )
)]

pub struct ApiDoc;

pub fn openapi_routes() -> Router {
    Router::new()
        .merge(SwaggerUi::new("/docs").url("/openapi.json", ApiDoc::openapi()))
        .route("/openapi.json", axum::routing::get(openapi_json))
        .route("/openapi.yaml", axum::routing::get(openapi_yaml))
}

async fn openapi_json() -> axum::response::Json<utoipa::openapi::OpenApi> {
    axum::response::Json(ApiDoc::openapi())
}

async fn openapi_yaml() -> Result<String, StatusCode> {
    serde_yaml::to_string(&ApiDoc::openapi())
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}