use std::{path::Path, process::Command};
use actix_web::web;
use bb8::{Pool, PooledConnection};
use bb8_tiberius::ConnectionManager;
// use dbase::{FieldIOError, FieldName, FieldType, FieldValue, FieldWriter, Record, TableWriterBuilder, WritableRecord};
use futures::StreamExt;
// use printpdf::{BuiltinFont, Mm, PdfDocument};
// use sailfish::TemplateOnce;
use tiberius::{numeric::Numeric};
use tokio::io::AsyncWriteExt;
use umya_spreadsheet::*;
use crate::contexts::model::{ActionResult, ReportRow};

use super::data_service::DataService;


// #[derive(TemplateOnce)]
// #[template(path = "../templates/order-data.stpl")]  // Template sailfish di folder templates/report.stpl
// struct ReportTemplate<'a> {
//     rows: &'a [ReportRow],
// }

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

    pub async fn export_to_xml_file<P: AsRef<Path>>(connection: web::Data<Pool<ConnectionManager>>, output_path: P) -> ActionResult<String, String> {
        let mut result = ActionResult::default();

        // 1. Ambil koneksi
        let mut conn: PooledConnection<ConnectionManager> = match connection.get().await {
            Ok(c) => c,
            Err(e) => {
                result.message = "Failed to get DB connection".into();
                result.error   = Some(e.to_string());
                return result;
            }
        };

        // 2. Query data
        let query = r#"
            SELECT Email, FullName, Age, Sex, Contact,
                ProductName, ProductCount, Price, IPAddress, LastUpdate
            FROM TempImport
        "#;
        let stream = match conn.query(query, &[]).await {
            Ok(r) => r.into_row_stream(),
            Err(e) => {
                result.message = "Failed to query".into();
                result.error   = Some(e.to_string());
                return result;
            }
        };

        // 3. Pastikan direktori ada
        let path = output_path.as_ref();
        if let Some(dir) = path.parent() {
            if let Err(e) = tokio::fs::create_dir_all(dir).await {
                result.message = "Failed to create dir".into();
                result.error   = Some(e.to_string());
                return result;
            }
        }

        // 4. Buat file
        let mut file = match tokio::fs::File::create(path).await {
            Ok(f) => f,
            Err(e) => {
                result.message = "Failed to create file".into();
                result.error   = Some(e.to_string());
                return result;
            }
        };

        // 5. Tulis header XML
        if let Err(e) = file.write_all(b"<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<Records>\n").await {
            result.message = "Failed to write XML header".into();
            result.error   = Some(e.to_string());
            return result;
        }

        // 6. Iterasi baris dan tulis setiap record sebagai elemen <Record>
        let mut rows = stream;
        while let Some(row_res) = rows.next().await {
            match row_res {
                Ok(row) => {
                    // ambil setiap kolom
                    let email        : &str               = row.get("Email").unwrap_or("");
                    let full_name    : &str               = row.get("FullName").unwrap_or("");
                    let age          : i32                = row.get("Age").unwrap_or(0);
                    let sex          : &str               = row.get("Sex").unwrap_or("");
                    let contact      : &str               = row.get("Contact").unwrap_or("");
                    let product_name : &str               = row.get("ProductName").unwrap_or("");
                    let product_count: i32                = row.get("ProductCount").unwrap_or(0);
                    let price: f64 = match row.try_get::<Numeric, _>("Price") {
                        Ok(Some(n)) => DataService::numeric_to_f64(&n).unwrap_or(0.0),
                        _ => 0.0,
                    };
                    let ip_address   : &str               = row.get("IPAddress").unwrap_or("");
                    // bangun XML untuk satu record
                    let xml = format!(
                        "  <Record>\n\
                        \t<Email>{}</Email>\n\
                        \t<FullName>{}</FullName>\n\
                        \t<Age>{}</Age>\n\
                        \t<Sex>{}</Sex>\n\
                        \t<Contact>{}</Contact>\n\
                        \t<ProductName>{}</ProductName>\n\
                        \t<ProductCount>{}</ProductCount>\n\
                        \t<Price>{:.2}</Price>\n\
                        \t<IPAddress>{}</IPAddress>\n\
                        </Record>\n",
                        Self::xml_escape(email),
                        Self::xml_escape(full_name),
                        age,
                        Self::xml_escape(sex),
                        Self::xml_escape(contact),
                        Self::xml_escape(product_name),
                        product_count,
                        price,
                        Self::xml_escape(ip_address),
                    );

                    if let Err(e) = file.write_all(xml.as_bytes()).await {
                        result.message = "Failed to write record".into();
                        result.error   = Some(e.to_string());
                        return result;
                    }
                }
                Err(e) => {
                    result.message = "Error fetching row".into();
                    result.error   = Some(e.to_string());
                    return result;
                }
            }
        }

        // 7. Tulis footer & selesai
        if let Err(e) = file.write_all(b"</Records>") .await {
            result.message = "Failed to write XML footer".into();
            result.error   = Some(e.to_string());
            return result;
        }

        result.result = true;
        result.message = "Export XML successful".into();
        result.data    = Some(path.to_string_lossy().to_string());
        result
    }

    pub async fn export_to_pdf_file<P: AsRef<Path>>(
        pool: web::Data<Pool<ConnectionManager>>,
        output_path: P,
    ) -> ActionResult<String, String> {
        let mut result = ActionResult::default();

        // 1️⃣ Ambil koneksi DB
        let mut conn = match pool.get().await {
            Ok(c) => c,
            Err(e) => {
                result.message = "Failed to get DB connection".into();
                result.error = Some(e.to_string());
                return result;
            }
        };

        // 2️⃣ Query data
        let query = r#"
            SELECT Email, FullName, Age, Sex, Contact,
                ProductName, ProductCount, Price, IPAddress, LastUpdate
            FROM TempImport
        "#;
        let mut stream = match conn.query(query, &[]).await {
            Ok(r) => r.into_row_stream(),
            Err(e) => {
                result.message = "Query failed".into();
                result.error = Some(e.to_string());
                return result;
            }
        };

        // 3️⃣ Kumpulkan data ke Vec<ReportRow>
        let mut rows = Vec::new();
        while let Some(row_res) = stream.next().await {
            match row_res {
                Ok(row) => {
                    // Mapping kolom ke struct
                    let email: String = row.get("Email").unwrap_or("").to_string();
                    let full_name: String = row.get("FullName").unwrap_or("").to_string();
                    let age: i32 = row.get("Age").unwrap_or(0);
                    let sex: String = row.get("Sex").unwrap_or("").to_string();
                    let contact: String = row.get("Contact").unwrap_or("").to_string();
                    let product_name: String = row.get("ProductName").unwrap_or("").to_string();
                    let product_count: i32 = row.get("ProductCount").unwrap_or(0);
                    let price: f64 = match row.try_get::<Numeric, _>("Price") {
                        Ok(Some(n)) => DataService::numeric_to_f64(&n).unwrap_or(0.0),
                        _ => 0.0,
                    };
                    let ip_address: String = row.get("IPAddress").unwrap_or("").to_string();
                    let last_update: String = row
                        .get::<chrono::NaiveDateTime, _>("LastUpdate")
                        .map(|dt| dt.to_string())
                        .unwrap_or_default();

                    rows.push(ReportRow {
                        email,
                        full_name,
                        age,
                        sex,
                        contact,
                        product_name,
                        product_count,
                        price,
                        ip_address,
                        last_update,
                    });
                }
                Err(e) => {
                    result.message = "Error reading row".into();
                    result.error = Some(e.to_string());
                    return result;
                }
            }
        }

        // 4️⃣ Render HTML pake Sailfish
        // let tpl = ReportTemplate { rows: &rows };
        // let html = match tpl.render_once() {
        //     Ok(h) => h,
        //     Err(e) => {
        //         result.message = "Failed to render template".into();
        //         result.error = Some(e.to_string());
        //         return result;
        //     }
        // };
        let html = "Hello world!";
        // 5️⃣ Simpan HTML sementara ke file
        let tmp_html_path = format!("templates/exports/{}.html", chrono::Utc::now().timestamp_millis());
        if let Err(e) = tokio::fs::write(&tmp_html_path, html).await {
            result.message = "Failed to write temporary HTML file".into();
            result.error = Some(e.to_string());
            return result;
        }

        // 6️⃣ Jalankan wkhtmltopdf di blocking thread agar gak blocking async runtime
        let output_path = output_path.as_ref().to_owned();
        let output_path_owned = output_path.to_owned();
        let tmp_html_clone = tmp_html_path.clone();
        let wk_res = tokio::task::spawn_blocking(move || {
            Command::new("wkhtmltopdf")
                .arg(&tmp_html_clone)
                .arg(output_path.clone())
                .status()
        })
        .await;

    println!("wk_res: {:#?}", wk_res);

        // Handle hasil eksekusi
        match wk_res {
            Ok(Ok(status)) if status.success() => {
                // Berhasil generate PDF
                // Hapus file HTML sementara
                let _ = tokio::fs::remove_file(&tmp_html_path);

                result.result = true;
                result.message = "PDF exported successfully".into();
                result.data = Some(output_path_owned.to_string_lossy().into());
                result
            }
            Ok(Ok(status)) => {
                result.message = "wkhtmltopdf exited with error".into();
                result.error = Some(format!("Exit code: {:?}", status.code()));
                result
            }
            Ok(Err(e)) => {
                result.message = "Failed to run wkhtmltopdf".into();
                result.error = Some(e.to_string());
                result
            }
            Err(e) => {
                result.message = "Failed to spawn blocking task".into();
                result.error = Some(e.to_string());
                result
            }
        }
    }

    pub async fn export_to_dbf_file<P: AsRef<Path>>(connection: web::Data<bb8::Pool<ConnectionManager>>,output_path: P) -> ActionResult<String, String> {
        let mut result = ActionResult::default();

        let path = output_path.as_ref();
        if let Some(dir) = path.parent() {
            if let Err(e) = tokio::fs::create_dir_all(dir).await {
                result.message = "Failed to create dir".into();
                result.error = Some(e.to_string());
                return result;
            }
        }

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

        result.result = true;
        result.data = Some(path.to_string_lossy().to_string());

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

    // helper sederhana untuk escape teks ke XML
    fn xml_escape(s: &str) -> String {
        s.replace("&", "&amp;")
        .replace("<", "&lt;")
        .replace(">", "&gt;")
        .replace("\"", "&quot;")
        .replace("'", "&apos;")
    }

}