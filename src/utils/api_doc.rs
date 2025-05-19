use actix_web::{get, web, HttpResponse, Responder};
use utoipa::{OpenApi, ToSchema};

use crate::contexts::{jwt_session::Claims, model::{ActionResult, ChangePasswordRequest, EmailRequest, HeaderParams, LoginRequest, NewNoteRequest, RegisterRequest, ResetPasswordRequest, TableDataParams}};

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
    tag = "1. Authentiacation"
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
    tag = "1. Authentiacation"
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
    tag = "1. Authentiacation"
)]
#[allow(dead_code)]
pub fn check_session_doc() {}

// Logout Docs
#[utoipa::path(post, path = "/api/v1/auth/logout", 
    responses(
        (status = 200, description = "Logout Success", body = ActionResult<String, String>)
    ),
    tag = "1. Authentiacation"
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
    tag = "1. Authentiacation"
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
    tag = "1. Authentiacation"
)]
#[allow(dead_code)]
pub fn reset_password_doc() {}

// Change password User Docs
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
    tag = "1. Authentiacation"
)]
#[allow(dead_code)]
pub fn change_password_doc() {}

// Contact Form Docs
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

// Create Library Docs
#[utoipa::path(
    post, 
    path = "/api/v1/library/create", 
    request_body = NewNoteRequest,
    summary = "Cek sesi login pengguna",
    description = "`Wajib login terlebih dahulu. Memerlukan token dari cookies` untuk mengecek sesi login pengguna",
    responses(
        (status = 200, description = "New notes created", body = ActionResult<Claims, String>, example = json!({
            "result": true, 
            "message": "New notes created"
        })),
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
    tag = "3. Library Endpoints"
)]
#[allow(dead_code)]
pub fn create_library_doc() {}

// Get Libraries Docs
#[utoipa::path(
    get,
    path = "/api/v1/library/get/{category}",
    params(
        ("category" = String, Path, description = "Category of notes"),
    ),
    summary = "Cek sesi login pengguna",
    description = "`Wajib login terlebih dahulu. Memerlukan token dari cookies` untuk mengecek sesi login pengguna",
    responses(
        (status = 200, description = "Get Libraries", body = ActionResult<Claims, String>, example = json!({
            "result": true,
            "message": "Retrieve libraries successfully",
            "data": [
                {
                    "NotesNID": 1,
                    "Title": "Library 1",
                    "Slug": "library-1",
                    "Category": "technology",
                    "Content_MD": "https://raw.githubusercontent.com/Snakesystem/docs/refs/heads/main/mssql-with-rust/README.md"
                },
                {
                    "NotesNID": 2,
                    "Title": "Library 2",
                    "Slug": "library-2",
                    "Category": "database",
                    "Content_MD": "https://raw.githubusercontent.com/Snakesystem/docs/refs/heads/main/postgres-with-rust/README.md"
                }
            ]
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
    tag = "3. Library Endpoints"
)]
#[allow(dead_code)]
pub fn get_libraries_doc() {}

// Get Single Docs
#[utoipa::path(
    get,
    path = "/api/v1/library/get-single/{slug}",
    params(
        ("slug" = String, Path, description = "Slug of notes"),
    ),
    summary = "Cek sesi login pengguna",
    description = "`Wajib login terlebih dahulu. Memerlukan token dari cookies` untuk mengecek sesi login pengguna",
    responses(
        (status = 200, description = "Get Single Library", body = ActionResult<Claims, String>, example = json!({
            "result": true,
            "message": "Retrieve library successfully",
            "data": {
                "NotesNID": 1,
                "Title": "Library 1",
                "Slug": "library-1",
                "Category": "technology",
                "Content_MD": "https://raw.githubusercontent.com/Snakesystem/docs/refs/heads/main/mssql-with-rust/README.md"
            },
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
    tag = "3. Library Endpoints"
)]
#[allow(dead_code)]
pub fn get_library_doc() {}

// Get header Docs
#[utoipa::path(
    get,
    path = "/api/v1/data/header",
    summary = "Get generic columns",
    description = "`Wajib login terlebih dahulu. Memerlukan token dari cookies` untuk mengecek sesi login pengguna",
    params(
        HeaderParams
    ),
    responses(
        (status = 200, description = "Check Session", body = ActionResult<Claims, String>, example = json!({
            "result": true,
            "message": "Data retrieved successfully",
            "data": [
                {
                    "field": "DataNID",
                    "filterControl": "input",
                    "sortable": true,
                    "title": "Data NID"
                },
                {
                    "field": "DataName",
                    "filterControl": "input",
                    "sortable": true,
                    "title": "Data Name"
                },
            ]
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
    tag = "4. Data Endpoints"
)]
#[allow(dead_code)]
pub fn get_header_docs(_: web::Query<HeaderParams>) {}

// Get Table Data Docs
#[utoipa::path(
    get,
    path = "/api/v1/data/get-table",
    summary = "Get generic columns",
    description = "`Wajib get header terlebih dahulu.` untuk mengecek header columns",
    params(
        TableDataParams
    ),
    responses(
        (status = 200, description = "Data retrieved successfully", example = json!({
            "totalNotFiltered": 222,
            "total": 222,
            "rows": [
                {
                "DataNID": 1,
                "DataID": "DATA-123",
                "DataName": "Jasa Keuangan Pasar Senggol",
                "DataDescription": "Jasa Keuangan Pasar Senggol",
                "LastUpdate": "2021-01-01"
                },
                {
                "DataNID": 2,
                "DataID": "DATA-124",
                "DataName": "Jasa Keuangan Pasar Kecil",
                "DataDescription": "Jasa Keuangan Pasar Kecil",
                "LastUpdate": "2021-01-01"
                },
                {
                "DataNID": 3,
                "DataID": "DATA-125",
                "DataName": "Jasa Keuangan Pasar Besar",
                "DataDescription": "Jasa Keuangan Pasar Besar",
                "LastUpdate": "2021-01-01"
                }
            ]
    
        })),
        (status = 500, description = "Internal Server Error", example = json!({
            "error": "Token error: 'Invalid column name 'AutoNID'.' on server S3 executing  on line 1 (code: 207, state: 1, class: 16)"
        })),
    ),
    tag = "4. Data Endpoints"
)]
#[allow(dead_code)]
pub fn get_table_data_docs(params: web::Query<TableDataParams>) {
    params.into_inner();
}

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

// Not Found Docs
#[utoipa::path(get, path = "/random-url/test",
    responses(
        (status = 404, description = "Not found", body = ActionResult<String, String>, example = json!({
            "result": false, 
            "message": "Not found", 
            "error": "Url '/random-url/test' not found. Please check the URL."
        }))
    ),
    tag = "5. Generic Endpoints"
)]
#[allow(dead_code)]
pub fn not_found_docs() {}

// Not Found Docs
#[utoipa::path(get, path = "/api/v1/generic/ws/",
    responses(
        (status = 200, description = "Web Socket Success", example = json!({
            "message": "on progress", 
        }))
    ),
    tag = "5. Generic Endpoints"
)]
#[allow(dead_code)]
pub fn ws_route_docs() {}

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
        contact_form_doc,
        create_library_doc,
        get_libraries_doc,
        get_library_doc,
        not_found_docs,
        get_header_docs,
        get_table_data_docs
    ),
    components(
        schemas(ActionResult<Claims, String>)
    ),
    tags(
        (name = "0. Application Default Endpoints", description = "Default path application endpoints"),
        (name = "1. Authentiacation", description = "Authentication related endpoints"),
        (name = "2. Email Endpoints", description = "Mailer to send email related endpoints"),
        (name = "3. Library Endpoints", description = "Library endpoints to manage library data for Snakesystem Library"),
        (name = "4. Data Endpoints", description = "Data endpoints to manage generic data"),
        (name = "5. Generic Endpoints", description = "Generic endpoints to manage reusable url"),
    )
)]

pub struct ApiDoc;