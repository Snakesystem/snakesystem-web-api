use actix_web::{error, web, HttpRequest, HttpResponse, Responder};
use bb8::Pool;
use bb8_tiberius::ConnectionManager;
use serde_json::json;
use tiberius::QueryStream;
use rand::{rng, Rng};

use crate::contexts::model::{ActionResult, Company};

pub struct GenericService;

impl GenericService {
    pub async fn get_company(connection: web::Data<Pool<ConnectionManager>>) -> ActionResult<Company, String> {
        let mut result = ActionResult::default();

        match connection.clone().get().await {
            Ok(mut conn) => {
                let query_result: Result<QueryStream, _> = conn
                    .query("SELECT CompanyID, CompanyName FROM Company", &[])
                    .await;
                match query_result {
                    Ok(rows) => {
                        if let Ok(Some(row)) = rows.into_row().await {
                            result.result = true;
                            result.message = "Company name".to_string();
                            result.data = Some(Company {
                                company_id: row
                                    .get::<&str, _>("CompanyID")
                                    .map_or_else(|| "".to_string(), |s| s.to_string()),
                                company_name: row
                                    .get::<&str, _>("CompanyName")
                                    .map_or_else(|| "".to_string(), |s| s.to_string()),
                            });
                            return result;
                        } else {
                            result.message = "No company found".to_string();
                            return result;
                        }
                    }
                    Err(e) => {
                        result.message = "Internal Server Error".to_string();
                        result.error = Some(e.to_string());
                        return result;
                    }
                }
            }
            Err(e) => {
                result.error = Some(e.to_string());
                return result;
            }
        }
    }

    pub fn json_error_handler(
        err: error::JsonPayloadError,
        _req: &actix_web::HttpRequest,
    ) -> actix_web::Error {
        let error_message = format!("Json deserialize error: {}", err);

        let result = ActionResult::<String, _> {
            // <- Ubah dari ActionResult<()> ke ActionResult<String>
            result: false,
            message: "Invalid Request".to_string(),
            error: Some(error_message), // <- Sekarang cocok karena `data: Option<String>`
            data: None,
        };

        error::InternalError::from_response(err, HttpResponse::BadRequest().json(result)).into()
    }

    pub async fn not_found(req: HttpRequest) -> impl Responder {
        HttpResponse::NotFound().json({
            json!({
                "result": false,
                "message": "Not Found",
                "error": format!("Url '{}' not found. Please check the URL.", req.path())
            })
        })
    }

    pub fn random_string(length: usize) -> String {
        const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
        let mut rng = rng();
    
        (0..length)
            .map(|_| {
                let idx = rng.random_range(0..CHARS.len());
                CHARS[idx] as char
            })
            .collect()
    }

    pub fn get_ip_address(req: &HttpRequest) -> String {
        req.headers()
            .get("X-Forwarded-For") // Jika pakai reverse proxy seperti Nginx
            .and_then(|ip| ip.to_str().ok())
            .map_or_else(
                || req.peer_addr()
                    .map(|addr| addr.ip().to_string())
                    .unwrap_or_else(|| "Unknown IP".to_string()),
                |ip| ip.to_string(),
            )
    }
}