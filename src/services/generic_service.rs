use actix_web::{error, web, HttpRequest, HttpResponse, Responder};
use bb8::Pool;
use bb8_tiberius::ConnectionManager;
use serde_json::json;
use tiberius::QueryStream;
use rand::{rng, Rng};

use crate::contexts::{model::{ActionResult, Company, MyRow}, socket::send_ws_event};

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

    pub fn json_error_handler(err: error::JsonPayloadError, _req: &actix_web::HttpRequest) -> actix_web::Error {
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

        if req.path() == "/docs" {
            return HttpResponse::Found()
            .append_header(("Location", "/docs/index.html"))
            .finish()
        }

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

    pub fn get_device_name(req: &HttpRequest) -> String {
        let test = req.headers()
            .get("X-Forwarded-Host")
            .and_then(|ua| ua.to_str().ok())
            .map_or_else(
                || "Unknown Device".to_string(),
                |ua| ua.to_string(),
            );

        return test
    }

    pub fn is_localhost_origin(req: &HttpRequest) -> bool {
        if let Some(origin) = req.headers().get("Origin") {
            if let Ok(origin_str) = origin.to_str() {
                return origin_str.starts_with("http://localhost");
            }
        }
        false
    }

    pub fn get_secret_key() -> [u8; 32] {
        let key_str = std::env::var("JWT_KEY").expect("JWT_KEY not set");
        let key_bytes = key_str.as_bytes();

        if key_bytes.len() != 32 {
            panic!("JWT_KEY must be exactly 32 bytes");
        }

        let mut key_array = [0u8; 32];
        key_array.copy_from_slice(&key_bytes[..32]);
        key_array
    }

    pub fn slugify(title: &str) -> String {
        title
            .to_lowercase()
            .chars()
            .map(|c| {
                if c.is_alphanumeric() {
                    c
                } else if c.is_whitespace() || c == '-' {
                    '-'
                } else {
                    '\0' // dibuang nanti
                }
            })
            .collect::<String>()
            .split('-') // hilangkan extra dash
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join("-")
    }
    
    pub fn sanitize_filename(filename: &str) -> String {
        filename
            .chars()
            .map(|c| {
                if c.is_ascii_alphanumeric() || c == '.' || c == '_' || c == '-' {
                    c
                } else {
                    '_'
                }
            })
            .collect()
    }

    pub async fn test_import_data(rows: Vec<MyRow>) {
        let total = rows.len();

        for (i, row) in rows.iter().enumerate() {
            // Simulasi pemrosesan data (misalnya simpan ke DB)
            println!("Import row: {:?}", row);

            // Kirim progress ke frontend
            let progress = json!({
                "current": i + 1,
                "total": total,
                "row": row.name, // contoh data tambahan
            });

            send_ws_event("import_progress", &progress);

            // Simulasi delay kalau mau lihat progress-nya jelas
            tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
        }

        // Kirim notifikasi selesai
        send_ws_event("import_done", &json!({
            "status": "done",
            "imported": total
        }));
    }

}