use std::path::{Path, PathBuf};
use futures::StreamExt;
use tokio_util::compat::TokioAsyncReadCompatExt;
use actix_web::web;
use bb8::Pool;
use bb8_tiberius::ConnectionManager;
use chrono::Utc;
use csv_async::AsyncReaderBuilder;
use tokio::{fs::File, io::{AsyncBufReadExt, BufReader}};

use crate::contexts::{connection::Transaction, model::ActionResult, socket::send_ws_event};

pub struct ImportService;

impl ImportService {
    pub async fn import_csv_from_file(file_path: PathBuf, connection: web::Data<Pool<ConnectionManager>>) -> ActionResult<String, String> {
        let mut result = ActionResult::default();
        let mut rowsaffected: u64 = 0;

        let total_count = match Self::count_csv_rows(&file_path).await {
            Ok(count) => count,
            Err(err) => {
                result.message = "File open error".to_string();
                result.error = Some(format!("Failed to open file: {}", err));
                return result;
            }
        };

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
                let file = match File::open(&file_path).await {
                    Ok(f) => f,
                    Err(err) => {
                        result.message = "File open error".to_string();
                        result.error = Some(format!("Failed to open file: {}", err));
                        return result;
                    }
                };

                let reader = file.compat(); // <-- Ini penting, convert ke futures::AsyncRead

                let mut rdr = AsyncReaderBuilder::new()
                    .has_headers(true)
                    .create_deserializer(reader);

                let mut records = rdr.deserialize::<(
                    String, String, i32, String, String, String, i32, f64, String,
                )>();

                while let Some(record) = records.next().await {
                    let (email, full_name, age, sex, contact, product_name, count, price, ip) =
                        match record {
                            Ok(row) => row,
                            Err(e) => {
                                result.message = "CSV parse error".to_string();
                                result.error = Some(format!("Failed to parse row: {}", e));
                                return result;
                            }
                        };

                    let query = r#"
                        INSERT INTO TempImport (
                            Email, FullName, Age, Sex, Contact, ProductName,
                            ProductCount, Price, IPAddress, LastUpdate
                        )
                        VALUES (@P1, @P2, @P3, @P4, @P5, @P6, @P7, @P8, @P9, @P10);
                    "#;

                    let now = Utc::now().naive_utc();

                    match conn.execute(query, &[&email, &full_name, &age, &sex, &contact, &product_name, &count, &price, &ip, &now]).await {
                        Ok(res) => {
                            rowsaffected += res.rows_affected().iter().sum::<u64>();

                            // bagian web socket
                            let progress = serde_json::json!({
                                "current": rowsaffected,
                                "total": total_count,
                                "message": result.message.clone()
                            });

                            send_ws_event("import_progress", &progress);
                        },
                        Err(e) => {
                            result.message = "Insert error".to_string();
                            result.error = Some(format!("Query failed: {}", e));
                            return result;
                        }
                    }
                }
            }
            None => {
                result.message = "Internal server error".to_string();
                result.error = Some("Failed to get connection from pool".to_string());
                return result;
            }
        }

        if rowsaffected > 0 {
            send_ws_event("import_done", serde_json::json!({
                "result": true,
                "imported": rowsaffected,
                "message": result.message.clone()
            }));
            if let Err(e) = trans.commit().await {
                result.message = "Failed to commit".to_string();
                result.error = Some(format!("Commit error: {}", e));
                return result;
            }
            result.result = true;
            result.message = format!("Berhasil insert {} baris.", rowsaffected);
        } else {
            send_ws_event("import_error", serde_json::json!({
                "result": false,
                "imported": rowsaffected,
                "message": result.message.clone(),
                "error": result.error.clone()
            }));
            trans.rollback().await.ok();
            result.message = "Tidak ada data yang di-insert.".to_string();
        }

        result
    }

    pub async fn count_csv_rows(file_path: &Path) -> Result<usize, std::io::Error> {
        let file = File::open(file_path).await?;
        let reader = BufReader::new(file);

        let mut lines = reader.lines();
        let mut count = 0;

        // Lewati header
        if let Some(_) = lines.next_line().await? {
            while let Some(_) = lines.next_line().await? {
                count += 1;
            }
        }

        Ok(count)
    }

}