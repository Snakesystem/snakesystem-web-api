use actix_web::{get, web, HttpResponse, Responder, Scope};
use bb8::Pool;
use bb8_tiberius::ConnectionManager;
use serde_json::json;

use crate::{contexts::model::{ActionResult, HeaderParams, ResultList, TableDataParams}, services::data_service::DataService};

pub fn data_scope() -> Scope {
    web::scope("/data")
        .service(get_header)
        .service(get_table_data)
}

#[get("/header")]
pub async fn get_header(pool: web::Data<Pool<ConnectionManager>>, params: web::Query<HeaderParams>) -> impl Responder {

    let result: ActionResult<Vec<serde_json::Value>, String> = DataService::get_header(pool, params.into_inner().tablename).await;

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

#[get("/get-table")]
async fn get_table_data(params: web::Query<TableDataParams>, pool: web::Data<Pool<ConnectionManager>>) -> impl Responder {

    let data: Result<ResultList, Box<dyn std::error::Error>> = DataService::get_table_data(params.into_inner(), pool).await;

    match data {
        Ok(response) => {
            return HttpResponse::Ok().json(response);
        },
        Err(e) => {
            return HttpResponse::InternalServerError().json(
                json!({"error": e.to_string()})
            );
        },
        
    }
}