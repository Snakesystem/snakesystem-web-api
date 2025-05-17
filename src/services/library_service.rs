use actix_web::{web, HttpRequest};
use bb8::Pool;
use bb8_tiberius::ConnectionManager;
use chrono::Utc;

use crate::contexts::{connection::Transaction, model::{ActionResult, NewNoteRequest}};

use super::{data_service::DataService, generic_service::GenericService};

pub struct LibraryService;

impl LibraryService {
    pub async fn create_library(req: HttpRequest, connection: web::Data<Pool<ConnectionManager>>, request: web::Json<NewNoteRequest>) -> ActionResult<String, String> {
        let mut result = ActionResult::default();

        let trans = match Transaction::begin(&connection).await {
            Ok(trans) => trans,
            Err(err) => {
                result.message = "Internal server error".to_string();
                result.error = Some(format!("Failed to begin transaction: {}", err));
                return result;
            }
        };

        match trans.conn.lock().await.as_mut() {
            Some(conn) => {
                let query = r#"
                    INSERT INTO Notes (NotesCategory, Title, Slug, Content_MD, IPAddress, LastUpdate)
                    VALUES (@P1, @P2, @P3, @P4, @P5, @P6);"#;
                
                if let Err(err) = conn.execute(query, &[
                    &request.category,
                    &request.title,
                    &request.slug,
                    &request.content_md,
                    &GenericService::get_ip_address(&req),
                    &Utc::now().naive_utc(),
                ]).await {
                    result.message = "Internal server error".to_string();
                    result.error = Some(format!("Query error: {}", err));
                } else {
                    result.result = true;
                    result.message = "Notes created successfully".to_string();
                }
            } 
            None => {
                result.message = "Internal server error".to_string();
                result.error = Some("Failed to get connection from pool".to_string());
            }
        }

        match result.result {
            true => {
                if let Err(err) = trans.commit().await {
                    result.message = "Internal server error".to_string();
                    result.error = Some(format!("Failed to commit transaction: {}", err));
                    return result;
                }
            }
            false => {
                if let Err(err) = trans.rollback().await {
                    result.message = "Internal server error".to_string();
                    result.error = Some(format!("Failed to rollback transaction: {}", err));
                    return result;
                }
            }
            
        }
        return result;
    }

    pub async fn get_libraries(connection: web::Data<Pool<ConnectionManager>>, category: String) -> ActionResult<Vec<serde_json::Value>, String> {
        let mut result: ActionResult<Vec<serde_json::Value>, String> = ActionResult::default();
        let mut conn = connection.get().await.unwrap();

        match conn.query(r#"SELECT * FROM Notes WHERE NotesCategory = @P1 ORDER BY LastUpdate DESC"#, &[&category]).await {
            Ok(rows) => {
                let data: Vec<serde_json::Value> = rows.into_results().await.unwrap().into_iter()
                    .flat_map(|r| r.into_iter())
                    .filter_map(|row| Some(DataService::row_to_json(&row)))
                    .collect();
                result.result = true;
                result.message = "Retrieved data successfully".to_string();
                result.data = Some(data);
            }
            Err(err) => {
                result.message = "Internal server error".to_string();
                result.error = Some(format!("Query error: {}", err));
            }
        }

        return result;
    }
}