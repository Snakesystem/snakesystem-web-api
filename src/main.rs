use actix_cors::Cors;
use actix_web::{http, web::{self, route, ServiceConfig}};
use contexts::connection::{create_pool, DbPool};
use handlers::{auth_handler::auth_scope, mail_handler::mail_scope, generic_handler::generic_scope};
use services::generic_service::GenericService;
use shuttle_actix_web::ShuttleActixWeb;
use utils::api_doc::{health_check, ApiDoc};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

mod contexts {
    pub mod connection;
    pub mod model;
    pub mod crypto;
    pub mod jwt_session;
}

mod services {
    pub mod generic_service;
    pub mod auth_service;
    pub mod mail_service;
}

mod handlers {
    pub mod generic_handler;
    pub mod auth_handler;
    pub mod mail_handler;
}

mod utils {
    pub mod validation;
    pub mod api_doc;
}

#[shuttle_runtime::main]
async fn main() -> ShuttleActixWeb<impl FnOnce(&mut ServiceConfig) + Send + Clone + 'static> {
    let db_pool: DbPool = create_pool("db12877").await.unwrap();

    let config = move |cfg: &mut ServiceConfig| {
        let cors = Cors::default()
            .allow_any_origin() // Atau pakai .allow_any_origin() dynamic app https only
            // .allowed_origin("http://localhost:5173") // url development
            // .allowed_origin("https://snakesystem.github.io") // url production
            .allowed_methods(vec!["GET", "POST", "OPTIONS"])
            .allowed_headers(vec![http::header::CONTENT_TYPE])
            .max_age(3600)
            .supports_credentials();
        
        cfg
        .service(health_check)
        .service(
            web::scope("/api/v1")
            .wrap(cors)
            .service(generic_scope())
            .service(auth_scope())
            .service(mail_scope())
        )
        .service(
            SwaggerUi::new("/docs/{_:.*}")
                .url("/api-docs/openapi.json", ApiDoc::openapi())
        )
        .app_data(web::Data::new(db_pool.clone()))
        .app_data(web::JsonConfig::default().error_handler(GenericService::json_error_handler))
        .default_service(route().to(GenericService::not_found));
    };

    Ok(config.into())
}
