use std::path::PathBuf;
use actix_multipart::Multipart;
use actix_web::{post, web, HttpResponse, Responder, Scope};
use bb8::Pool;
use bb8_tiberius::ConnectionManager;
use futures::StreamExt;
use tokio::{fs::File, io::AsyncWriteExt};

use crate::services::{generic_service::GenericService, import_service::ImportService};

pub fn import_scope() -> Scope {
    
    web::scope("/import")
        .service(import_csv_handler)
        .service(import_txt_handler)
}

#[post("/csv")]
pub async fn import_csv_handler(mut payload: Multipart, connection: web::Data<Pool<ConnectionManager>>) -> impl Responder {
    let tmp_path = PathBuf::from("./templates/uploads");
    if let Err(e) = tokio::fs::create_dir_all(&tmp_path).await {
        return HttpResponse::InternalServerError().json(serde_json::json!({
            "result": false,
            "message": format!("Failed to create temp dir: {}", e)
        }));
    }

    let mut file_path = None;

    while let Some(field_res) = payload.next().await {
        let mut field = match field_res {
            Ok(field) => field,
            Err(_) => {
                return HttpResponse::BadRequest().json(serde_json::json!({
                    "result": false,
                    "message": "Failed to read field"
                }));
            }
        };

        let file_name = field.content_disposition()
            .and_then(|cd| cd.get_filename().map(GenericService::sanitize_filename))
            .unwrap_or_else(|| "upload.csv".to_string());

        let path = tmp_path.join(&file_name);

        match File::create(&path).await {
            Ok(mut f) => {
                while let Some(chunk) = field.next().await {
                    match chunk {
                        Ok(data) => {
                            if let Err(e) = f.write_all(&data).await {
                                return HttpResponse::InternalServerError().json(serde_json::json!({
                                    "result": false,
                                    "message": format!("Failed to write file: {}", e)
                                }));
                            }
                        }
                        Err(_) => {
                            return HttpResponse::BadRequest().json(serde_json::json!({
                                "result": false,
                                "message": "Failed to read chunk"
                            }));
                        }
                    }
                }
                file_path = Some(path);
            }
            Err(e) => {
                return HttpResponse::InternalServerError().json(serde_json::json!({
                    "result": false,
                    "message": format!("Failed to create file: {}", e)
                }));
            }
        }
    }

    if let Some(csv_file) = file_path {
        let connection_clone = connection.clone();
        let file_clone = csv_file.clone();

        // 🚀 Jalankan import di background
        tokio::spawn(async move {
            let _ = ImportService::import_csv_from_file(file_clone.clone(), connection_clone).await;
            let _ = tokio::fs::remove_file(file_clone).await;
        });

        // ⏱️ Balas langsung, proses jalan di background
        return HttpResponse::Ok().json(serde_json::json!({
            "result": true,
            "status": "processing",
            "message": "File berhasil diupload, sedang diproses."
        }));
    }

    HttpResponse::BadRequest().json(serde_json::json!({
        "result": false,
        "message": "File not found"
    }))
}

#[post("/txt")]
pub async fn import_txt_handler(mut payload: Multipart, connection: web::Data<Pool<ConnectionManager>>) -> impl Responder {
    let tmp_dir = PathBuf::from("./templates/uploads");
    if let Err(e) = tokio::fs::create_dir_all(&tmp_dir).await {
        return HttpResponse::InternalServerError().json(serde_json::json!({
            "result": false,
            "message": format!("Failed to create temp dir: {}", e)
        }));
    }

    let mut file_path = None;

    // Simpan file upload
    while let Some(field_res) = payload.next().await {
        let mut field = match field_res {
            Ok(f) => f,
            Err(_) => {
                return HttpResponse::BadRequest().json(serde_json::json!({
                    "result": false,
                    "message": "Failed to read field"
                }));
            }
        };

        let file_name = field.content_disposition()
            .and_then(|cd| cd.get_filename().map(GenericService::sanitize_filename))
            .unwrap_or_else(|| "upload.txt".to_string());

        let path = tmp_dir.join(&file_name);
        match File::create(&path).await {
            Ok(mut f) => {
                while let Some(chunk) = field.next().await {
                    let data = chunk.map_err(|_| ()).unwrap();
                    f.write_all(&data).await.map_err(|_| ()).unwrap();
                }
                file_path = Some(path);
            }
            Err(e) => {
                return HttpResponse::InternalServerError().json(serde_json::json!({
                    "result": false,
                    "message": format!("Failed to create file: {}", e)
                }));
            }
        }
    }

    // Jika sukses upload, jalankan background import
    if let Some(txt_file) = file_path {
        let conn_clone = connection.clone();
        let file_clone = txt_file.clone();
        tokio::spawn(async move {
            let _ = ImportService::import_txt_from_file(file_clone.clone(), conn_clone).await;
            let _ = tokio::fs::remove_file(file_clone).await;
        });

        return HttpResponse::Ok().json(serde_json::json!({
            "result": true,
            "status": "processing",
            "message": "File TXT berhasil diupload, sedang diproses."
        }));
    }

    HttpResponse::BadRequest().json(serde_json::json!({
        "result": false,
        "message": "File not found"
    }))
}