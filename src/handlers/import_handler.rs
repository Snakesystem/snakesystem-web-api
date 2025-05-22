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
        .service(import_xlsx_handler)
        .service(import_dbf_handler)
        .service(import_xml_handler)
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

#[post("/xlsx")]
pub async fn import_xlsx_handler(mut payload: Multipart, connection: web::Data<Pool<ConnectionManager>>) -> impl Responder {
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
            .unwrap_or_else(|| "upload.xlsx".to_string());

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

    // Jalankan import
    if let Some(xlsx_file) = file_path {
        let conn_clone = connection.clone();
        let file_clone = xlsx_file.clone();
        tokio::spawn(async move {
            let _ = ImportService::import_xlsx_from_file(file_clone.clone(), conn_clone, true).await;
            let _ = tokio::fs::remove_file(file_clone).await;
        });

        return HttpResponse::Ok().json(serde_json::json!({
            "result": true,
            "status": "processing",
            "message": "File XLSX berhasil diupload, sedang diproses."
        }));
    }

    HttpResponse::BadRequest().json(serde_json::json!({
        "result": false,
        "message": "File not found"
    }))
}

#[post("/dbf")]
pub async fn import_dbf_handler(mut payload: Multipart, connection: web::Data<Pool<ConnectionManager>>) -> impl Responder {
    // 1. Buat temp dir
    let tmp_dir = PathBuf::from("./templates/uploads");
    if let Err(e) = tokio::fs::create_dir_all(&tmp_dir).await {
        return HttpResponse::InternalServerError().json(serde_json::json!({
            "result": false,
            "message": format!("Failed to create temp dir: {}", e)
        }));
    }

    // 2. Simpan file upload
    let mut file_path = None;
    while let Some(field) = payload.next().await {
        let mut field = match field {
            Ok(f) => f,
            Err(_) => break,
        };
        let filename = field
            .content_disposition()
            .and_then(|cd| cd.get_filename().map(|s| GenericService::sanitize_filename(s)))
            .unwrap_or_else(|| "upload.dbf".into());
        let path = tmp_dir.join(&filename);
        let f = File::create(&path).await.map_err(|e| {
            eprintln!("File create error: {}", e);
        }).ok();
        if let Some(mut f) = f {
            while let Some(chunk) = field.next().await {
                let data = chunk.unwrap();
                f.write_all(&data).await.unwrap();
            }
            file_path = Some(path);
        }
    }

    // 3. Proses DBF di background
    if let Some(file_clone) = file_path {
        let conn = connection.clone();
        tokio::spawn(async move {
            // Buka DBF dengan crate `dbase`
            let _ = ImportService::import_dbf_from_file(file_clone.clone(), conn).await;
            let _ = tokio::fs::remove_file(file_clone).await;
        });

        HttpResponse::Ok().json(serde_json::json!({
            "result": true,
            "status": "processing",
            "message": "File DBF berhasil diupload, sedang diproses."
        }))
    } else {
        HttpResponse::BadRequest().json(serde_json::json!({
            "result": false,
            "message": "Tidak ada file DBF ditemukan"
        }))
    }
}

#[post("/xml")]
pub async fn import_xml_handler(mut payload: Multipart, connection: web::Data<Pool<ConnectionManager>>) -> impl Responder {
    // Buat direktori upload kalau belum ada
    let tmp_dir = PathBuf::from("./templates/uploads");
    if let Err(e) = tokio::fs::create_dir_all(&tmp_dir).await {
        return HttpResponse::InternalServerError().json(serde_json::json!({
            "result": false,
            "message": format!("Failed to create temp dir: {}", e)
        }));
    }

    let mut file_path: Option<PathBuf> = None;

    // Simpan file XML yang di‐upload
    while let Some(field_res) = payload.next().await {
        let mut field = match field_res {
            Ok(f) => f,
            Err(_) => {
                return HttpResponse::BadRequest().json(serde_json::json!({
                    "result": false,
                    "message": "Failed to read multipart field"
                }));
            }
        };

        // Ambil nama file, default .xml
        let file_name = field
            .content_disposition()
            .and_then(|cd| cd.get_filename().map(GenericService::sanitize_filename))
            .unwrap_or_else(|| "upload.xml".to_string());

        let path = tmp_dir.join(&file_name);
        match File::create(&path).await {
            Ok(mut f) => {
                while let Some(chunk) = field.next().await {
                    let data = match chunk {
                        Ok(bytes) => bytes,
                        Err(_) => break,
                    };
                    if f.write_all(&data).await.is_err() {
                        break;
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

    // Jika ada file, jalankan import di background
    if let Some(xml_file) = file_path {
        let conn_clone = connection.clone();
        let file_clone = xml_file.clone();
        tokio::spawn(async move {
            // Panggil service import_xml_from_file
            let _ = ImportService::import_xml_from_file(file_clone.clone(), conn_clone).await;
            let _ = tokio::fs::remove_file(file_clone).await;
        });

        HttpResponse::Ok().json(serde_json::json!({
            "result": true,
            "status": "processing",
            "message": "File XML berhasil diupload, sedang diproses."
        }))
    } else {
        HttpResponse::BadRequest().json(serde_json::json!({
            "result": false,
            "message": "No XML file uploaded"
        }))
    }
}