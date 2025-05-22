use std::path::{Path, PathBuf};
use actix_web::web;
use bb8::{Pool, PooledConnection};
use bb8_tiberius::ConnectionManager;
use dbase::{FieldName, FieldType, FieldValue, Record, TableWriterBuilder};
use futures::StreamExt;
use tiberius::numeric::Numeric;
use tokio::io::AsyncWriteExt;
use umya_spreadsheet::*;
use crate::contexts::model::ActionResult;

use super::data_service::DataService;

pub struct ExportService;

impl ExportService {
    pub async fn export_to_csv_file<P: AsRef<Path>>(connection: web::Data<Pool<ConnectionManager>>, output_path: P) -> ActionResult<String, String> {
        let mut result = ActionResult::default();

        // Ambil koneksi
        let mut conn: PooledConnection<ConnectionManager> = match connection.get().await {
            Ok(c) => c,
            Err(e) => {
                result.message = "Failed to get DB connection".into();
                result.error = Some(e.to_string());
                return result;
            }
        };

        let query = r#"
            SELECT 
                Email, FullName, Age, Sex, Contact,
                ProductName, ProductCount, Price, IPAddress, LastUpdate
            FROM TempImport
        "#;

        // Jalankan query dan langsung ambil hasil
        let rows_result = conn.query(query, &[]).await;
        let mut stream = match rows_result {
            Ok(r) => r.into_row_stream(),
            Err(e) => {
                result.message = "Failed to query".into();
                result.error = Some(e.to_string());
                return result;
            }
        };

        let mut csv = String::from("Email,FullName,Age,Sex,Contact,ProductName,ProductCount,Price,IPAddress\r\n");

        while let Some(row_result) = stream.next().await {
            match row_result {
                Ok(row) => {
                    let email: &str = row.get("Email").unwrap_or("");
                    let full_name: &str = row.get("FullName").unwrap_or("");
                    let age: i32 = row.get("Age").unwrap_or(0);
                    let sex: &str = row.get("Sex").unwrap_or("");
                    let contact: &str = row.get("Contact").unwrap_or("");
                    let product_name: &str = row.get("ProductName").unwrap_or("");
                    let product_count: i32 = row.get("ProductCount").unwrap_or(0);
                    let price: f64 = match row.try_get::<Numeric, _>("Price") {
                        Ok(Some(n)) => DataService::numeric_to_f64(&n).unwrap_or(0.0),
                        _ => 0.0,
                    };
                    let ip: &str = row.get("IPAddress").unwrap_or("");
                    // let last_update: chrono::NaiveDateTime = row.get("LastUpdate").unwrap_or(chrono::Utc::now().naive_utc());

                    // Tambahkan ke CSV
                    let escape = |s: &str| {
                        if s.contains(',') || s.contains('"') {
                            format!("\"{}\"", s.replace('"', "\"\""))
                        } else {
                            s.to_string()
                        }
                    };

                    csv.push_str(&format!(
                        "{},{},{},{},{},{},{},{},{}\r\n",
                        escape(email),
                        escape(full_name),
                        age,
                        escape(sex),
                        escape(contact),
                        escape(product_name),
                        product_count,
                        price,
                        escape(ip),
                    ));
                }
                Err(e) => {
                    result.message = "Error fetching row".into();
                    result.error = Some(e.to_string());
                    return result;
                }
            }
        }

        // Simpan ke file
        let path = output_path.as_ref();
        if let Some(dir) = path.parent() {
            if let Err(e) = tokio::fs::create_dir_all(dir).await {
                result.message = "Failed to create dir".into();
                result.error = Some(e.to_string());
                return result;
            }
        }

        match tokio::fs::write(path, csv).await {
            Ok(_) => {
                result.result = true;
                result.message = "Export successful".into();
                result.data = Some(path.to_string_lossy().to_string());
                result
            }
            Err(e) => {
                result.message = "Failed to write file".into();
                result.error = Some(e.to_string());
                result
            }
        }
    }

    pub async fn export_to_txt_file<P: AsRef<Path>>(connection: web::Data<Pool<ConnectionManager>>, output_path: P) -> ActionResult<String, String> {
        let mut result = ActionResult::default();

        let mut conn: PooledConnection<ConnectionManager> = match connection.get().await {
            Ok(c) => c,
            Err(e) => {
                result.message = "Failed to get DB connection".into();
                result.error = Some(e.to_string());
                return result;
            }
        };

        let query = r#"SELECT 
            Email, FullName, Age, Sex, Contact, ProductName, ProductCount, Price, IPAddress, lastUpdate 
            FROM TempImport"#;
        
        let row_result = conn.query(query, &[]).await;

        let mut stream = match row_result {
            Ok(r) => r.into_row_stream(),
            Err(e) => {
                result.message = "Failed to query".into();
                result.error = Some(e.to_string());
                return result;
            }
        };

        let path = output_path.as_ref();
        if let Some(dir) = path.parent() {
            if let Err(e) = tokio::fs::create_dir_all(dir).await {
                result.message = "Failed to create dir".into();
                result.error = Some(e.to_string());
                return result;
            }
        }

        let mut file = match tokio::fs::File::create(path).await {
            Ok(f) => f,
            Err(e) => {
                result.message = "Failed to create file".into();
                result.error = Some(e.to_string());
                return result;
            }
        };

        if let Err(e) = file
            .write_all(b"Email|FullName|Age|Sex|Contact|ProductName|ProductCount|Price|IPAddress\r\n")
            .await
        {
            result.message = "Failed to write header".into();
            result.error = Some(e.to_string());
            return result;
        }

        while let Some(row_result) = stream.next().await {
            match row_result {
                Ok(row) => {
                    let email: &str = row.get("Email").unwrap_or("");
                    let full_name: &str = row.get("FullName").unwrap_or("");
                    let age: i32 = row.get("Age").unwrap_or(0);
                    let sex: &str = row.get("Sex").unwrap_or("");
                    let contact: &str = row.get("Contact").unwrap_or("");
                    let product_name: &str = row.get("ProductName").unwrap_or("");
                    let product_count: i32 = row.get("ProductCount").unwrap_or(0);
                    let price: f64 = match row.try_get::<Numeric, _>("Price") {
                        Ok(Some(n)) => DataService::numeric_to_f64(&n).unwrap_or(0.0),
                        _ => 0.0,
                    };
                    let ip: &str = row.get("IPAddress").unwrap_or("");

                    let line = format!(
                        "{}|{}|{}|{}|{}|{}|{}|{}|{}\r\n",
                        email,
                        full_name,
                        age,
                        sex,
                        contact,
                        product_name,
                        product_count,
                        price,
                        ip,
                    );

                    if let Err(e) = file.write_all(line.as_bytes()).await {
                        result.message = "Failed to write row".into();
                        result.error = Some(e.to_string());
                        return result;
                    }
                }
                Err(e) => {
                    result.message = "Error fetching row".into();
                    result.error = Some(e.to_string());
                    return result;
                }
            }
        }

        result.result = true;
        result.message = "Export TXT successful".into();
        result.data = Some(path.to_string_lossy().to_string());

        result
    }

    pub async fn export_to_xlsx_file<P: AsRef<Path>>(connection: web::Data<bb8::Pool<ConnectionManager>>, output_path: P) -> ActionResult<String, String> {
        let mut result = ActionResult::default();

        // Ambil koneksi DB
        let mut conn = match connection.get().await {
            Ok(c) => c,
            Err(e) => {
                result.message = "Failed to get DB connection".into();
                result.error = Some(e.to_string());
                return result;
            }
        };

        // Query data
        let query = r#"
            SELECT 
                Email, FullName, Age, Sex, Contact,
                ProductName, ProductCount, Price, IPAddress
            FROM TempImport
        "#;

        let rows_result = conn.query(query, &[]).await;
        let mut stream = match rows_result {
            Ok(r) => r.into_row_stream(),
            Err(e) => {
                result.message = "Failed to query".into();
                result.error = Some(e.to_string());
                return result;
            }
        };

        // Buat workbook baru
        let mut book = new_file();
        let sheet_name = "order-data";
        let _ = book.new_sheet(sheet_name);

        // println!("Sheet name: {}, sheet: {:?}", sheet_name, sheet);

        // Header
        let headers = [
            "Email", "FullName", "Age", "Sex", "Contact",
            "ProductName", "ProductCount", "Price", "IPAddress"
        ];

        for (col, header) in headers.iter().enumerate() {
            let col_letter = Self::column_index_to_letter(col);
            let cell_name = format!("{}1", col_letter); // Baris tetap 1
            book.get_sheet_by_name_mut(sheet_name).unwrap().get_cell_mut(&*cell_name).set_value(header.to_string());
        }

        // Data rows
        let mut row_idx = 2u32;

        while let Some(row_result) = stream.next().await {
            match row_result {
                Ok(row) => {
                    let values: Vec<String> = vec![
                        row.get::<&str, _>("Email").unwrap_or("").to_string(),
                        row.get::<&str, _>("FullName").unwrap_or("").to_string(),
                        row.get::<i32, _>("Age").unwrap_or(0).to_string(),
                        row.get::<&str, _>("Sex").unwrap_or("").to_string(),
                        row.get::<&str, _>("Contact").unwrap_or("").to_string(),
                        row.get::<&str, _>("ProductName").unwrap_or("").to_string(),
                        row.get::<i32, _>("ProductCount").unwrap_or(0).to_string(),
                        match row.try_get::<Numeric, _>("Price") {
                            Ok(Some(n)) => DataService::numeric_to_f64(&n)
                                .map(|v| v.to_string())
                                .unwrap_or("0.0".to_string()),
                            _ => "0.0".to_string(),
                        },
                        row.get::<&str, _>("IPAddress").unwrap_or("").to_string()
                    ];

                    // println!("Values: {:?}", values);
                    for (col_idx, value) in values.iter().enumerate() {
                        let col_letter = Self::column_index_to_letter(col_idx);
                        let cell_ref = format!("{}{}", col_letter, row_idx);
                        book.get_sheet_by_name_mut("order-data") // ganti dengan nama sheet kamu
                            .unwrap()
                            .get_cell_mut(&*cell_ref)
                            .set_value(value);
                    }

                    row_idx += 1;
                }
                Err(e) => {
                    println!("Error fetching row: {}", e);
                    result.message = "Error fetching row".into();
                    result.error = Some(e.to_string());
                    return result;
                }
            }
        }

        // Simpan file
        let path = output_path.as_ref();
        if let Some(dir) = path.parent() {
            if let Err(e) = tokio::fs::create_dir_all(dir).await {
                result.message = "Failed to create directory".into();
                result.error = Some(e.to_string());
                return result;
            }
        }

        let _ = book.remove_sheet_by_name("Sheet1");
        if let Err(e) = writer::xlsx::write(&book, path) {
            result.message = "Failed to write XLSX".into();
            result.error = Some(e.to_string());
            return result;
        }

        result.result = true;
        result.message = "Export XLSX successful".into();
        result.data = Some(path.to_string_lossy().to_string());
        result
    }

    pub async fn export_dbf_to_file(
        file_path: PathBuf,
        connection: web::Data<Pool<ConnectionManager>>,
    ) -> ActionResult<String, String> {
        let mut result = ActionResult::default();

        let conn = match connection.get().await {
            Ok(c) => c,
            Err(e) => {
                result.message = "Gagal mendapatkan koneksi database".into();
                result.error = Some(e.to_string());
                return result;
            }
        };

        // Ambil data dari tabel TempImport
        let rows = match conn
            .query("SELECT Email, FullName, Age, Sex, Contact, ProductName, ProductCount, Price, IPAddress FROM TempImport", &[])
            .await
        {
            Ok(r) => r,
            Err(e) => {
                result.message = "Gagal query data".into();
                result.error = Some(format!("Query error: {}", e));
                return result;
            }
        };

        // Definisikan struktur kolom DBF
        let fields = vec![
            (FieldName::try_from("EMAIL").unwrap(), FieldType::Character(Some(100))),
            (FieldName::try_from("FULLNAME").unwrap(), FieldType::Character(Some(100))),
            (FieldName::try_from("AGE").unwrap(), FieldType::Numeric(Some(10), Some(0))),
            (FieldName::try_from("SEX").unwrap(), FieldType::Character(Some(10))),
            (FieldName::try_from("CONTACT").unwrap(), FieldType::Character(Some(50))),
            (FieldName::try_from("PRODUCTNAM").unwrap(), FieldType::Character(Some(100))),
            (FieldName::try_from("PRODUCTCOU").unwrap(), FieldType::Numeric(Some(10), Some(0))),
            (FieldName::try_from("PRICE").unwrap(), FieldType::Numeric(Some(18), Some(2))),
            (FieldName::try_from("IPADDRESS").unwrap(), FieldType::Character(Some(50))),
        ];

        // Buat file output
        let writer = match tokio::fs::File::create(&file_path) {
            Ok(f) => f,
            Err(e) => {
                result.message = "Gagal membuat file DBF".into();
                result.error = Some(e.to_string());
                return result;
            }
        };

        let mut table_writer = TableWriterBuilder::from_writer(writer)
            .add_fields(fields.clone())
            .build()
            .unwrap();

        for row in rows {
            let record = Record::new()
                .insert("EMAIL", FieldValue::Character(Some(row.get::<_, String>(0).unwrap_or_default())))
                .insert("FULLNAME", FieldValue::Character(Some(row.get::<_, String>(1).unwrap_or_default())))
                .insert("AGE", FieldValue::Numeric(Some(row.get::<_, i32>(2).unwrap_or(0) as f64)))
                .insert("SEX", FieldValue::Character(Some(row.get::<_, String>(3).unwrap_or_default())))
                .insert("CONTACT", FieldValue::Character(Some(row.get::<_, String>(4).unwrap_or_default())))
                .insert("PRODUCTNAM", FieldValue::Character(Some(row.get::<_, String>(5).unwrap_or_default())))
                .insert("PRODUCTCOU", FieldValue::Numeric(Some(row.get::<_, i32>(6).unwrap_or(0) as f64)))
                .insert("PRICE", FieldValue::Numeric(Some(row.get::<_, f64>(7).unwrap_or(0.0))))
                .insert("IPADDRESS", FieldValue::Character(Some(row.get::<_, String>(8).unwrap_or_default())));

            if let Err(e) = table_writer.write_record(&record) {
                result.message = "Gagal menulis record ke DBF".into();
                result.error = Some(format!("DBF write error: {}", e));
                return result;
            }
        }

        result.result = true;
        result.message = "Export DBF berhasil".into();
        result
    }

    fn column_index_to_letter(mut col_index: usize) -> String {
        let mut col_letter = String::new();
        col_index += 1; // Biar 0 = A, 1 = B, dst
        while col_index > 0 {
            let rem = (col_index - 1) % 26;
            col_letter.insert(0, (b'A' + rem as u8) as char);
            col_index = (col_index - 1) / 26;
        }
        col_letter
    }

}