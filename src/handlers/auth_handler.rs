use actix_web::{cookie::{time, Cookie, SameSite}, get, post, web, HttpRequest, HttpResponse, Responder, Scope};
use bb8::Pool;
use bb8_tiberius::ConnectionManager;
use serde_json::json;
use crate::{
    contexts::{jwt_session::{create_jwt, validate_jwt, Claims}, 
    model::{ActionResult, ChangePasswordRequest, LoginRequest, RegisterRequest, ResetPasswordRequest}}, 
    services::{auth_service::AuthService, generic_service::GenericService}
};

const APP_NAME: &str = "snakesystem-web-api";

pub fn auth_scope() -> Scope {
    
    web::scope("/auth")
        .service(login)
        .service(register)
        .service(check_session)
        .service(logout)
        .service(activation_user)
        .service(forget_password)
        .service(change_password)
}

#[post("/login")]
async fn login(req: HttpRequest, connection: web::Data<Pool<ConnectionManager>>, request: web::Json<LoginRequest>) -> impl Responder {

    let mut result: ActionResult<Claims, _> = AuthService::login(connection.clone(), request.into_inner(), req.clone(), APP_NAME).await;

    let token_cookie = req.cookie("snakesystem").map(|c| c.value().to_string()).unwrap_or_default();

    let is_localhost = GenericService::is_localhost_origin(&req); // pass `req` into your handler

    match result {
        response if response.error.is_some() => {
            HttpResponse::InternalServerError().json(response)
        }, // Jika error, HTTP 500
        response if response.result => {
            if let Some(user) = &response.data {
                // ✅ Buat token JWT
                match create_jwt(user.clone()) {
                    Ok(token) => {
                        // ✅ Simpan token dalam cookie
                        result = AuthService::check_session(connection, user.clone(), token.clone(), token_cookie.clone(), false, false, false).await;

                        // ✅ Jika berhasil, kembalikan JSON response
                        if !result.result {
                            return HttpResponse::InternalServerError().json(result);
                        }
                            
                        let cookie = Cookie::build("snakesystem", token_cookie.is_empty().then(|| token.clone()).unwrap_or(token_cookie.clone()))
                            .path("/")
                            .http_only(true)
                            .same_site(SameSite::Lax)
                            .secure(!is_localhost) // Ubah ke `true` jika pakai HTTPS
                            .expires(time::OffsetDateTime::now_utc() + time::Duration::days(1))
                            .finish();

                        return HttpResponse::Ok()
                            .cookie(cookie)
                            .json(response);
                    }
                    Err(err) => {
                        println!("❌ Failed to create JWT: {}", err);
                        return HttpResponse::InternalServerError().json(response);
                    }
                }
            }

            HttpResponse::BadRequest().json(response) // Jika tidak ada user, return 400
        },
        response => HttpResponse::BadRequest().json(response), // Jika gagal login, HTTP 400
    }
}

#[get("/session")]
async fn check_session(req: HttpRequest, connection: web::Data<Pool<ConnectionManager>>) -> impl Responder {

    let mut result: ActionResult<Claims, _> = ActionResult::default();

    // Ambil cookie "token"
    let token_cookie = req.cookie("snakesystem");

    // Cek apakah token ada di cookie
    let token = match token_cookie {
        Some(cookie) => cookie.value().to_string(),
        None => {
            result.error = Some("Token not found".to_string());
            return HttpResponse::Unauthorized().json(result);
        }
    };

    // Validate token
    match validate_jwt(&token) {
        Ok(claims) => {
            result = AuthService::check_session(connection, claims.clone(), token.clone(), token.clone(), false, false, true).await;

            match result {
                response if response.error.is_some() => {
                    HttpResponse::InternalServerError().json(response)
                },
                response if response.result => {
                    
                    HttpResponse::Ok().json({
                        json!({
                            "result": response.result,
                            "message": response.message,
                            "data": Some(claims.clone())
                        })
                    })
                },
                response => HttpResponse::BadRequest().json(response), // Jika gagal login, HTTP 400
            }
        },
        Err(err) => {
            result.error = Some(err.to_string());
            HttpResponse::Unauthorized().json(result)
        },
    }
}

#[post("/logout")]
async fn logout() -> impl Responder {

    // Hapus cookie dengan setting expired date
    let cookie = Cookie::build("token", "")
        .path("/")
        .http_only(true)
        .same_site(SameSite::None)
        .secure(true) // Ubah ke true jika pakai HTTPS
        .max_age(time::Duration::days(-1)) // Set expired
        .finish();

    HttpResponse::Ok()
        .cookie(cookie) // Hapus cookie dengan expired
        .json(serde_json::json!({
            "result": true,
            "message": "Logout successful, cookie deleted"
        }))
}

#[post("/register")]
async fn register(req: HttpRequest, pool: web::Data<Pool<ConnectionManager>>, mut request: web::Json<RegisterRequest>) -> impl Responder {

    request.app_ipaddress = GenericService::get_ip_address(&req);

    let result: ActionResult<(), _> = AuthService::register(pool, request.into_inner()).await;

    match result {
        response if response.error.is_some() => {
            HttpResponse::InternalServerError().json(response)
        }, // Jika error, HTTP 500
        response if response.result => HttpResponse::Ok().json(response), // Jika berhasil, HTTP 200
        response => HttpResponse::BadRequest().json(response), // Jika gagal, HTTP 400
    }
}

#[get("/activation/{otp_link}")]
async fn activation_user(pool: web::Data<Pool<ConnectionManager>>, otp_link: web::Path<String>) -> impl Responder {

    let result: ActionResult<(), _> = AuthService::activation_user(pool, otp_link.into_inner()).await;

    match result {
        response if response.error.is_some() => {
            HttpResponse::InternalServerError().json(response)
        }, // Jika error, HTTP 500
        response if response.result => HttpResponse::Ok().json(response), // Jika berhasil, HTTP 200
        response => HttpResponse::BadRequest().json(response), // Jika gagal, HTTP 400
    }
}

#[post("/reset-password")]
async fn forget_password(pool: web::Data<Pool<ConnectionManager>>, request: web::Json<ResetPasswordRequest>) -> impl Responder {

    let result: ActionResult<(), _> = AuthService::forget_password(pool, request.into_inner()).await;

    match result {
        response if response.error.is_some() => {
            HttpResponse::InternalServerError().json(response)
        }, // Jika error, HTTP 500
        response if response.result => HttpResponse::Ok().json(response), // Jika berhasil, HTTP 200
        response => HttpResponse::BadRequest().json(response), // Jika gagal, HTTP 400
    }
}

#[post("/change-password")]
async fn change_password(pool: web::Data<Pool<ConnectionManager>>, request: web::Json<ChangePasswordRequest>) -> impl Responder {

    let result: ActionResult<(), _> = AuthService::change_password(pool, request.into_inner()).await;

    match result {
        response if response.error.is_some() => {
            HttpResponse::InternalServerError().json(response)
        }, // Jika error, HTTP 500
        response if response.result => HttpResponse::Ok().json(response), // Jika berhasil, HTTP 200
        response => HttpResponse::BadRequest().json(response), // Jika gagal, HTTP 400
    }
}