use actix_web::{get, web::{self, route, ServiceConfig}, HttpResponse, Responder};
use contexts::connection::{create_pool, DbPool};
use handlebars::Handlebars;
use handlers::{auth_handler::auth_scope, generic_handler::generic_scope};
use serde_json::{json, Value};
use services::generic_service::GenericService;
use reqwest::Client;
use shuttle_actix_web::ShuttleActixWeb;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;
use utoipa::ToSchema;

mod contexts {
    pub mod connection;
    pub mod model;
    pub mod crypto;
    pub mod jwt_session;
}

mod services {
    pub mod generic_service;
    pub mod auth_service;
}

mod handlers {
    pub mod generic_handler;
    pub mod auth_handler;
}

mod utils {
    pub mod validation;
}

#[derive(serde::Serialize, ToSchema)]
struct HealthCheckResponse {
    message: String,
}

#[utoipa::path(
    get,
    path = "/",
    responses(
        (status = 200, description = "Health Check Success", body = HealthCheckResponse)
    )
)]
#[get("/")]
async fn health_check() -> impl Responder {
    HttpResponse::Ok().json(HealthCheckResponse {
        message: "Welcome to the snakesystem app!".to_string(),
    })
}

#[derive(OpenApi)]
#[openapi(
    paths(
        health_check,
    ),
    components(
        schemas(HealthCheckResponse)
    ),
    tags(
        (name = "Health", description = "Health Check Endpoints")
    )
)]
struct ApiDoc;

#[get("/docs")]
pub async fn docs(hb: web::Data<Handlebars<'_>>) -> impl Responder {
    // 1. URL raw GitHub untuk docs.json
    let url = "https://raw.githubusercontent.com/Snakesystem/snakesystem-web-api/refs/heads/main/templates/docs.json";

    // 2. Membuat client HTTP untuk melakukan request
    let client = Client::new();

    // 3. Mengunduh file JSON dari GitHub
    let response = client.get(url)
        .send()
        .await
        .map_err(|err| {
            eprintln!("Failed to fetch file: {}", err);
            HttpResponse::InternalServerError().body("Failed to fetch docs.json")
        }).unwrap();

    if !response.status().is_success() {
        return HttpResponse::InternalServerError().body("Failed to fetch docs.json");
    }

    // 4. Mengambil konten JSON dari response
    let json_content: Value = response.json().await.map_err(|err| {
        eprintln!("Failed to parse JSON: {}", err);
        HttpResponse::InternalServerError().body("Failed to parse docs.json")
    }).unwrap();

    // 5. Pastikan request dan response di-serialize dengan benar
    let mut data = json_content.clone();
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

    // 6. Render menggunakan Handlebars
    let body = hb.render("docs", &data).unwrap();

    // 7. Return HTML sebagai response
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
        .service(
            web::scope("/api/v1")
            .service(generic_scope())
            .service(auth_scope())
        )
        .service(
            SwaggerUi::new("/swagger-ui/{_:.*}")
                .url("/api-docs/openapi.json", ApiDoc::openapi())
        )
        .app_data(web::Data::new(db_pool.clone()))
        .app_data(handlebars_ref.clone())
        .app_data(web::JsonConfig::default().error_handler(GenericService::json_error_handler))
        .default_service(route().to(GenericService::not_found));
    };

    Ok(config.into())
}
