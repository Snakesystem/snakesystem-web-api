use actix_web::{HttpResponse, Responder, get};
use utoipa::{OpenApi, ToSchema};

use crate::contexts::{jwt_session::Claims, model::{ActionResult, ChangePasswordRequest, EmailRequest, LoginRequest, RegisterRequest, ResetPasswordRequest}};

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
    ),
    tag = "1. Auth"
)]
#[allow(dead_code)]
pub fn login_doc() {}

// Register Docs
#[utoipa::path(post, path = "/api/v1/auth/register", request_body = RegisterRequest,
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
    ),
    tag = "1. Auth"
)]
#[allow(dead_code)]
pub fn register_doc() {}

// Check Session Docs
#[utoipa::path(
    get,
    path = "/api/v1/auth/session",
    summary = "Cek sesi login pengguna",
    description = "`Wajib login terlebih dahulu. Memerlukan token dari cookies` untuk mengecek sesi login pengguna",
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
    tag = "1. Auth"
)]
#[allow(dead_code)]
pub fn check_session_doc() {}

// Logout Docs
#[utoipa::path(post, path = "/api/v1/auth/logout", 
    responses(
        (status = 200, description = "Logout Success", body = ActionResult<String, String>)
    ),
    tag = "1. Auth"
)]
#[allow(dead_code)]
pub fn logout_doc() {}

// Activation User Docs
#[utoipa::path(
    get,
    path = "/api/v1/auth/activation/{otp_link}",
    params(
        ("otp_link" = String, Path, description = "Link OTP aktivasi user")
    ),
    responses(
        (status = 200, description = "Aktivasi berhasil", body = ActionResult<String, String>, example = json!({
            "result": true,
            "message": "Akun berhasil diaktivasi",
        })),
        (status = 400, description = "OTP tidak valid", body = ActionResult<String, String>, example = json!({
            "result": false,
            "message": "OTP invalid",
            "error": "Bad Request"
        })),
        (status = 500, description = "Gagal aktivasi", body = ActionResult<String, String>, example = json!({
            "result": false,
            "message": "Gagal aktivasi akun",
            "error": "Internal Server Error"
        }))
    ),
    tag = "1. Auth"
)]
#[allow(dead_code)]
pub fn activation_user_doc() {}

// Forget password User Docs
#[utoipa::path(
    post,
    path = "/api/v1/auth/reset-password",
    request_body = ResetPasswordRequest,
    responses(
        (status = 200, description = "Aktivasi berhasil", body = ActionResult<String, String>, example = json!({
            "result": true,
            "message": "Akun berhasil diaktivasi",
        })),
        (status = 400, description = "OTP tidak valid", body = ActionResult<String, String>, example = json!({
            "result": false,
            "message": "OTP invalid",
            "error": "Bad Request"
        })),
        (status = 500, description = "Gagal aktivasi", body = ActionResult<String, String>, example = json!({
            "result": false,
            "message": "Gagal aktivasi akun",
            "error": "Internal Server Error"
        }))
    ),
    tag = "1. Auth"
)]
#[allow(dead_code)]
pub fn reset_password_doc() {}

// Forget password User Docs
#[utoipa::path(
    post,
    path = "/api/v1/auth/change-password",
    request_body = ChangePasswordRequest,
    responses(
        (status = 200, description = "Aktivasi berhasil", body = ActionResult<String, String>, example = json!({
            "result": true,
            "message": "Akun berhasil diaktivasi",
        })),
        (status = 400, description = "OTP tidak valid", body = ActionResult<String, String>, example = json!({
            "result": false,
            "message": "OTP invalid",
            "error": "Bad Request"
        })),
        (status = 500, description = "Gagal aktivasi", body = ActionResult<String, String>, example = json!({
            "result": false,
            "message": "Gagal aktivasi akun",
            "error": "Internal Server Error"
        }))
    ),
    tag = "1. Auth"
)]
#[allow(dead_code)]
pub fn change_password_doc() {}

// Forget password User Docs
#[utoipa::path(
    post,
    path = "/api/v1/email/contact",
    request_body = EmailRequest,
    responses(
        (status = 200, description = "Succes sent email", body = ActionResult<String, String>, example = json!({
            "result": true,
            "message": "Email sent successfully!",
        })),
        (status = 400, description = "Bad request response", body = ActionResult<String, String>, example = json!({
            "result": false,
            "message": "Recipient not found",
            "error": "Bad Request"
        })),
        (status = 500, description = "Internet server error", body = ActionResult<String, String>, example = json!({
            "result": false,
            "message": "Email failed to send",
            "error": "Internal Server Error"
        }))
    ),
    tag = "2. Email Endpoints"
)]
#[allow(dead_code)]
pub fn contact_form_doc() {}

// Health Check Docs
#[utoipa::path(
    get,
    path = "/",
    responses(
        (status = 200, description = "Health Check Success", body = HealthCheckResponse, example = json!(HealthCheckResponse { message: "Welcome to the snakesystem app!".to_string(), }))
    ),
    tag = "0. Application Default Endpoints"
)]

#[get("/")]
pub async fn health_check() -> impl Responder {
    HttpResponse::Ok().json(HealthCheckResponse {
        message: "Welcome to the snakesystem app!".to_string(),
    })
}

#[derive(OpenApi)]
#[openapi(
    info(
        title = "Snakesystem API",
        description = "Dokumentasi untuk RESTful API SnakeSystem.\n\nSilakan gunakan token JWT untuk mengakses endpoint yang dilindungi.",
        version = "1.0.0"
    ),
    paths(
        health_check,
        login_doc,
        register_doc,
        reset_password_doc,
        change_password_doc,
        check_session_doc,
        logout_doc,
        activation_user_doc,
        contact_form_doc
    ),
    components(
        schemas(ActionResult<Claims, String>)
    ),
    tags(
        (name = "0. Application Default Endpoints", description = "Default path application endpoints"),
        (name = "1. Auth", description = "Authentication related endpoints"),
        (name = "2. Email Endpoints", description = "Mailer to send email related endpoints"),
    )
)]

pub struct ApiDoc;