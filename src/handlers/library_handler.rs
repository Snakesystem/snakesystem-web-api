use actix_web::{get, post, web, HttpRequest, HttpResponse, Responder, Scope};
use bb8::Pool;
use bb8_tiberius::ConnectionManager;
use validator::Validate;
use crate::{
    contexts::model::{ActionResult, NewNoteRequest, Notes}, 
    services::{generic_service::GenericService, library_service::LibraryService}
};

pub fn library_scope() -> Scope {
    web::scope("/library")
        .service(create_libary)
        .service(get_libraries)
        .service(get_library)
}

#[post("/create")]
async fn create_libary(req: HttpRequest, connection: web::Data<Pool<ConnectionManager>>, request: web::Json<NewNoteRequest>) -> impl Responder {
    if let Err(err) = request.validate() {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "result": false,
            "message": "Invalid request",
            "error": err
        }));
    }

    // ambil ownership dan ubah
    let mut request = request.into_inner();
    if request.slug.is_none() || request.slug.as_ref().unwrap().trim().is_empty() {
        request.slug = Some(GenericService::slugify(&request.title));
    }

    let result: ActionResult<String, String> =
        LibraryService::create_library(req, connection, web::Json(request)).await;

    if !result.result {
        return HttpResponse::InternalServerError().json(result);
    }

    HttpResponse::Ok().json(result)
}

#[get("/get/{category}")]
async fn get_libraries(connection: web::Data<Pool<ConnectionManager>>, category: web::Path<String>,) -> impl Responder {

    if category.is_empty() {
        return HttpResponse::BadRequest().json(serde_json::json!({"result": false, "message": "Category is empty"}));
    }

    let result: ActionResult<Vec<serde_json::Value>, String> = LibraryService::get_libraries(connection, category.into_inner()).await;
    if !result.result {
        return HttpResponse::InternalServerError().json(result);
    }
    HttpResponse::Ok().json(result)
}

#[get("/get-single/{category}")]
async fn get_library(connection: web::Data<Pool<ConnectionManager>>, slug: web::Path<String>,) -> impl Responder {

    if slug.is_empty() {
        return HttpResponse::BadRequest().json(serde_json::json!({"result": false, "message": "Slug is empty"}));
    }

    let result: ActionResult<Notes, String> = LibraryService::get_library(connection, slug.into_inner()).await;
    if !result.result {
        return HttpResponse::InternalServerError().json(result);
    }
    HttpResponse::Ok().json(result)
}