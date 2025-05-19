use actix_cors::Cors;
use actix_web::{http, web::{self, route, ServiceConfig}};
use contexts::connection::{create_pool, DbPool};
use handlers::{auth_handler::auth_scope, data_handler::data_scope, generic_handler::generic_scope, library_handler::library_scope, mail_handler::mail_scope};
use services::generic_service::GenericService;
use shuttle_actix_web::ShuttleActixWeb;
use shuttle_runtime::SecretStore;
use utils::api_doc::{health_check, ApiDoc};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

mod contexts {
    pub mod connection;
    pub mod model;
    pub mod crypto;
    pub mod jwt_session;
    pub mod socket;
}

mod services {
    pub mod generic_service;
    pub mod auth_service;
    pub mod mail_service;
    pub mod library_service;
    pub mod data_service;
}

mod handlers {
    pub mod generic_handler;
    pub mod auth_handler;
    pub mod mail_handler;
    pub mod library_handler;
    pub mod data_handler;
}

mod utils {
    pub mod validation;
    pub mod api_doc;
}

#[shuttle_runtime::main]
async fn main(#[shuttle_runtime::Secrets] secrets: SecretStore) -> ShuttleActixWeb<impl FnOnce(&mut ServiceConfig) + Send + Clone + 'static> {
    dotenvy::dotenv().ok();

    let db_server = secrets.get("DATABASE_SERVER").expect("secret was not found");
    let db_user = secrets.get("DATABASE_USER").expect("secret was not found");
    let db_password = secrets.get("DATABASE_PASSWORD").expect("secret was not found");

    let db_pool: DbPool = create_pool(db_server.as_str(), db_user.as_str(), db_password.as_str(), "db12877").await.unwrap();

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
            .service(library_scope())
            .service(data_scope())
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
