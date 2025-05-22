use std::path::PathBuf;

use actix_web::{get, HttpResponse, Responder, web, Scope};
use bb8::Pool;
use bb8_tiberius::ConnectionManager;

use crate::{contexts::model::ActionResult, services::export_service::ExportService};

pub fn export_scope() -> Scope {
    
    web::scope("/export")
        .service(download_csv_handler)
        .service(download_txt_handler)
        .service(download_xlsx_handler)
}

#[get("/csv")]
pub async fn download_csv_handler(connection: web::Data<Pool<ConnectionManager>>) -> impl Responder {
    // Tentukan path output
    let output_path = PathBuf::from("./exports/tempimport.csv");

    // Panggil service
    let res: ActionResult<String, String> =
        ExportService::export_to_csv_file(connection.clone(), output_path.clone()).await;

    if !res.result {
        return HttpResponse::InternalServerError().json(res);
    }

    // Baca file dan kirim sebagai download
    match tokio::fs::read(output_path).await {
        Ok(bytes) => HttpResponse::Ok()
            .content_type("text/csv; charset=utf-8")
            .append_header(("Content-Disposition", "attachment; filename=\"tempimport.csv\""))
            .body(bytes),
        Err(e) => HttpResponse::InternalServerError().json(ActionResult {
            result: false,
            message: "Failed to read CSV file".into(),
            data: "".into(),
            error: Some(e.to_string()),
        }),
    }
}

#[get("/txt")]
pub async fn download_txt_handler(connection: web::Data<Pool<ConnectionManager>>) -> impl Responder {
    // Tentukan path output
    let output_path = PathBuf::from("./exports/tempimport.txt");

    // Panggil service
    let res: ActionResult<String, String> =
        ExportService::export_to_txt_file(connection.clone(), output_path.clone()).await;

    if !res.result {
        return HttpResponse::InternalServerError().json(res);
    }

    // Baca file dan kirim sebagai download
    match tokio::fs::read(output_path).await {
        Ok(bytes) => HttpResponse::Ok()
            .content_type("text/plain; charset=utf-8")
            .append_header(("Content-Disposition", "attachment; filename=\"tempimport.txt\""))
            .body(bytes),
        Err(e) => HttpResponse::InternalServerError().json(ActionResult {
            result: false,
            message: "Failed to read TXT file".into(),
            data: "".into(),
            error: Some(e.to_string()),
        }),
    }
}

#[get("/xlsx")]
pub async fn download_xlsx_handler(connection: web::Data<Pool<ConnectionManager>>) -> impl Responder {
    // Tentukan path output
    let output_path = PathBuf::from("./exports/tempimport.xlsx");

    // Panggil service
    let res: ActionResult<String, String> =
        ExportService::export_to_xlsx_file(connection.clone(), output_path.clone()).await;

    if !res.result {
        return HttpResponse::InternalServerError().json(res);
    }

    // Baca file dan kirim sebagai download
    match tokio::fs::read(output_path).await {
        Ok(bytes) => HttpResponse::Ok()
            .content_type("application/vnd.openxmlformats-officedocument.spreadsheetml.sheet; charset=utf-8")
            .append_header(("Content-Disposition", "attachment; filename=\"tempimport.xlsx\""))
            .body(bytes),
        Err(e) => HttpResponse::InternalServerError().json(ActionResult {
            result: false,
            message: "Failed to read TXT file".into(),
            data: "".into(),
            error: Some(e.to_string()),
        }),
    }
}