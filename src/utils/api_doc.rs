use actix_web::{HttpResponse, Responder, get};
use utoipa::{OpenApi, ToSchema};

use crate::contexts::{jwt_session::Claims, model::{ActionResult, LoginRequest}};

#[derive(serde::Serialize, ToSchema)]
struct HealthCheckResponse {
    message: String,
}

// Login Docs
#[utoipa::path(post, path = "/api/v1/auth/login", request_body = LoginRequest,
    responses(
        (status = 200, description = "Check Session", body = ActionResult<Claims, String>, example = json!({"result": true, "message": "Login Success", "data": {
            "user_id": "1",
            "username": "admin",
            "email": "LXh4N@example.com",
            "company_id": "SS",
            "company_name": "Snake System Tech"
        }})),
        (status = 401, description = "Unauthorized", body = ActionResult<String, String>, example = json!({
            "result": false, 
            "message": "Unauthorized", 
            "error": "Unauthorized"
        })),
        (status = 500, description = "Internal Server Error", body = ActionResult<String, String>, example = json!({
            "result": false, 
            "message": "User not found", 
            "error": "Internal Server Error"
        })),
        (status = 400, description = "Bad Request", body = ActionResult<String, String>, example = json!({
            "result": false, 
            "message": "Token not found", 
            "error": "Bad Request"
        }))
    )
)]
#[allow(dead_code)]
pub fn login_doc() {}

// Check Session Docs
#[utoipa::path(
    get,
    path = "/api/v1/auth/session",
    responses(
        (status = 200, description = "Check Session", body = ActionResult<Claims, String>, example = json!({
            "result": true,
            "message": "Session active",
            "data": {
                "user_id": "1",
                "username": "admin",
                "email": "admin@example.com"
            }
        })),
        (status = 401, description = "Unauthorized", body = ActionResult<String, String>, example = json!({
            "result": false,
            "message": "Unauthorized",
            "error": "Unauthorized"
        })),
        (status = 500, description = "Internal Server Error", body = ActionResult<String, String>, example = json!({
            "result": false,
            "message": "Token has expired",
            "error": "Internal Server Error"
        })),
        (status = 400, description = "Bad Request", body = ActionResult<String, String>, example = json!({
            "result": false,
            "message": "Token not found",
            "error": "Bad Request"
        }))
    ),
)]
#[allow(dead_code)]
pub fn check_session_doc() {}

// Logout Docs
#[utoipa::path(post, path = "/api/v1/auth/logout", 
    responses(
        (status = 200, description = "Logout Success", body = ActionResult<String, String>)
    )
)]
#[allow(dead_code)]
pub fn logout_doc() {}

// Health Check Docs
#[utoipa::path(
    get,
    path = "/",
    responses(
        (status = 200, description = "Health Check Success", body = HealthCheckResponse, example = json!(HealthCheckResponse { message: "Welcome to the snakesystem app!".to_string(), }))
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
        login_doc,
        check_session_doc,
        logout_doc,
    ),
    components(
        schemas(ActionResult<Claims, String>)
    ),
    tags(
        (name = "Auth", description = "Authentication API")
    )
)]

pub struct ApiDoc;