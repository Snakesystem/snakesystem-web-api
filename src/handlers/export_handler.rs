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
        .service(download_xml_handler)
        .service(download_pdf_handler)
        .service(download_emails)
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

#[get("/xml")]
pub async fn download_xml_handler(connection: web::Data<Pool<ConnectionManager>>) -> impl Responder {
    // Tentukan path output
    let output_path = PathBuf::from("./exports/tempimport.xml");

    // Panggil service
    let res: ActionResult<String, String> =
        ExportService::export_to_xml_file(connection.clone(), output_path.clone()).await;

    if !res.result {
        return HttpResponse::InternalServerError().json(res);
    }

    // Baca file dan kirim sebagai download
    match tokio::fs::read(output_path).await {
        Ok(bytes) => HttpResponse::Ok()
            .content_type("application/xml; charset=utf-8")
            .append_header(("Content-Disposition", "attachment; filename=\"tempimport.xml\""))
            .body(bytes),
        Err(e) => HttpResponse::InternalServerError().json(ActionResult {
            result: false,
            message: "Failed to read DBF file".into(),
            data: "".into(),
            error: Some(e.to_string()),
        }),
    }
}

#[get("/pdf")]
pub async fn download_pdf_handler(connection: web::Data<Pool<ConnectionManager>>) -> impl Responder {
    let output_path = PathBuf::from("./exports/tempimport.pdf");

    let res = ExportService::export_to_pdf_file(connection.clone(), output_path.clone()).await;

    if !res.result {
        return HttpResponse::InternalServerError().json(res);
    }

    match tokio::fs::read(output_path).await {
        Ok(bytes) => HttpResponse::Ok()
            .content_type("application/pdf")
            .append_header(("Content-Disposition", "attachment; filename=\"tempimport.pdf\""))
            .body(bytes),
        Err(e) => HttpResponse::InternalServerError().json(ActionResult {
            result: false,
            message: "Failed to read PDF file".into(),
            data: "".into(),
            error: Some(e.to_string()),
        }),
    }
}

#[get("/download/emails")]
async fn download_emails() -> Result<HttpResponse, actix_web::Error> {
    let emails: Vec<String> = (0..1000)
        .map(|i| format!("user{}@example.com", i))
        .collect();

    let content = emails.join("\n");

    Ok(HttpResponse::Ok()
        .content_type("text/plain")
        .append_header(("Content-Disposition", "attachment; filename=\"emails.txt\""))
        .body(content))
}