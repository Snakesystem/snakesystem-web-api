use std::fs;
use actix_web::{get, web::{self, route, ServiceConfig}, HttpResponse, Responder};
use contexts::connection::{create_pool, DbPool};
use handlebars::Handlebars;
use handlers::generic_handler::generic_scope;
use serde_json::{json, Value};
use services::generic_service::GenericService;
use shuttle_actix_web::ShuttleActixWeb;

mod contexts {
    pub mod connection;
    pub mod model;
}

mod services {
    pub mod generic_service;
}

mod handlers {
    pub mod generic_handler;
}

#[get("/")]
async fn health_check() -> impl Responder {
    HttpResponse::Ok().json({
        json!({
            "message": "Welcome to the snakesystem app!"
        })
    })
}

#[get("/docs")]
pub async fn docs(hb: web::Data<Handlebars<'_>>) -> impl Responder {
    // 1. Baca file JSON yang berisi dokumentasi API
    fs::copy("./templates/docs.json", "./target/docs.json").expect("Failed to copy docs.json");
    let json_content = fs::read_to_string("./templates/docs.json")
        .expect("Failed to read docs.json");

    // 2. Parse ke Value
    let mut data: Value = serde_json::from_str(&json_content)
        .expect("Invalid JSON format");

    // 3. Pastikan request dan response di-serialize dengan benar
    if let Some(endpoints) = data.get_mut("endpoints") {
        for endpoint in endpoints.as_array_mut().unwrap() {
            if let Some(request) = endpoint.get_mut("request") {
                *request = json!(serde_json::to_string(request).unwrap());
            }
            if let Some(response) = endpoint.get_mut("response") {
                *response = json!(serde_json::to_string(response).unwrap());
            }
        }
    }

    // 4. Render menggunakan Handlebars
    let body = hb.render("docs", &data).unwrap();

    // 5. Return HTML sebagai response
    HttpResponse::Ok().content_type("text/html").body(body)
}

#[shuttle_runtime::main]
async fn main() -> ShuttleActixWeb<impl FnOnce(&mut ServiceConfig) + Send + Clone + 'static> {
    let db_pool: DbPool = create_pool("db12877").await.unwrap();

    let mut handlebars = Handlebars::new();

    // 1. Embed template directly into the binary using include_str!
    let docs_template = include_str!(".././templates/docs.mustache");

    // 2. Register the template with Handlebars
    handlebars
        .register_template_string("docs", docs_template)
        .unwrap();

    let handlebars_ref = web::Data::new(handlebars);

    let config = move |cfg: &mut ServiceConfig| {
        cfg
        .service(health_check)
        .service(docs)
        .service(web::scope("/api/v1").service(generic_scope()))
        .app_data(web::Data::new(db_pool.clone()))
        .app_data(handlebars_ref.clone())
        .app_data(web::JsonConfig::default().error_handler(GenericService::json_error_handler))
        .default_service(route().to(GenericService::not_found));
    };

    Ok(config.into())
}
