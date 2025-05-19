use actix_web::{get, post, web, HttpRequest, HttpResponse, Responder, Scope};
use actix_web_actors::ws;
use bb8::Pool;
use bb8_tiberius::ConnectionManager;

use crate::{contexts::{model::{ActionResult, Company, MyRow}, socket::WsSession}, services::generic_service::GenericService};

pub fn generic_scope() -> Scope {
    web::scope("/generic")
        .service(get_company)
        .service(ws_route)
}

#[get("/company")]
pub async fn get_company(pool: web::Data<Pool<ConnectionManager>>) -> impl Responder {

    let result: ActionResult<Company, _> = GenericService::get_company(pool).await;

    match result {
        response if response.error.is_some() => {
            HttpResponse::InternalServerError().json(response)
        }, 
        response if response.result => {
            HttpResponse::Ok().json(response)
        }, 
        response => {
            HttpResponse::BadRequest().json(response)
        }
    }
}

#[get("/ws/")]
pub async fn ws_route(req: HttpRequest, stream: web::Payload) -> actix_web::Result<HttpResponse> {
    ws::start(WsSession::new(), &req, stream)
}

#[post("/import")]
pub async fn test_import_data(body: web::Json<Vec<MyRow>>) -> impl Responder {
    // Langsung clone data biar tidak kena borrow
    let rows = body.into_inner();

    // Jalankan import async (tapi tidak blokir response)
    tokio::spawn(async move {
        GenericService::test_import_data(rows).await;
    });

    HttpResponse::Ok().json({
        serde_json::json!({
            "status": "import_started"
        })
    })
}