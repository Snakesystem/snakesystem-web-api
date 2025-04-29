use actix_web::{HttpResponse, Responder, get};
use utoipa::{OpenApi, ToSchema};

use crate::contexts::{jwt_session::Claims, model::ActionResult};

#[derive(serde::Serialize, ToSchema)]
struct HealthCheckResponse {
    message: String,
}

#[utoipa::path(
    get,
    path = "/",
    responses(
        (status = 200, description = "Health Check Success", body = HealthCheckResponse)
    )
)]

#[get("/")]
pub async fn health_check() -> impl Responder {
    HttpResponse::Ok().json(HealthCheckResponse {
        message: "Welcome to the snakesystem app!".to_string(),
    })
}

#[derive(OpenApi)]
#[openapi(
    paths(
        health_check,
        crate::handlers::auth_handler::login,
        crate::handlers::auth_handler::check_session,
        crate::handlers::auth_handler::logout,
    ),
    components(
        schemas(ActionResult<Claims, String>)
    )
)]

pub struct ApiDoc;