use std::{fs, path::{Path, PathBuf}};
use calamine::{open_workbook_auto, DataType, Reader};
use dbase::FieldValue;
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
                "message": "Insert error",
                "error": result.error.clone()
            }));
            trans.rollback().await.ok();
            result.message = "Tidak ada data yang di-insert.".to_string();
        }

        result
    }

    pub async fn import_txt_from_file(file_path: PathBuf, connection: web::Data<Pool<ConnectionManager>>) -> ActionResult<String, String> {
        let mut result = ActionResult::default();
        let mut rowsaffected: u64 = 0;

        // Hitung total baris
        let total_count = match Self::count_txt_lines(&file_path, false).await {
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

                let mut reader = BufReader::new(file).lines();
                let mut delimiter: Option<char> = None;
                // let mut header_column_count = 0;
                let mut line_number = 0;

                while let Some(line_res) = reader.next_line().await.transpose() {
                    let line = match line_res {
                        Ok(l) => l.trim().to_string(),
                        Err(e) => {
                            result.message = "TXT parse error".to_string();
                            result.error = Some(format!("Failed to read line: {}", e));
                            return result;
                        }
                    };

                    if line.is_empty() {
                        continue;
                    }

                    line_number += 1;

                    if delimiter.is_none() {
                        delimiter = Self::detect_delimiter(&line);
                        if delimiter.is_none() {
                            result.message = "Delimiter tidak dikenali".to_string();
                            result.error = Some("Gunakan , ; atau | sebagai pemisah.".to_string());
                            return result;
                        }

                        // header_column_count = line.split(delimiter.unwrap()).count();
                        continue; // skip header
                    }

                    let delimiter = delimiter.unwrap();
                    let fields: Vec<&str> = line.split(delimiter).collect();

                    if fields.len() != 9 {
                        result.message = format!("Baris {} harus punya 9 kolom", line_number);
                        result.error = Some(format!("Ditemukan {} kolom, seharusnya 9.", fields.len()));
                        return result;
                    }

                    let email        = fields[0].trim();
                    let full_name    = fields[1].trim();
                    let age: i32     = match fields[2].trim().parse() {
                        Ok(a) => a,
                        Err(_) => {
                            result.message = format!("Baris {}: age bukan angka", line_number);
                            result.error = Some(format!("Invalid number: {}", fields[2]));
                            return result;
                        }
                    };
                    let sex          = fields[3].trim();
                    let contact      = fields[4].trim();
                    let product_name = fields[5].trim();
                    let product_count: i32 = match fields[6].trim().parse() {
                        Ok(p) => p,
                        Err(_) => {
                            result.message = format!("Baris {}: product count bukan angka", line_number);
                            result.error = Some(format!("Invalid number: {}", fields[6]));
                            return result;
                        }
                    };
                    let price: f64 = match fields[7].trim().parse() {
                        Ok(p) => p,
                        Err(_) => {
                            result.message = format!("Baris {}: price bukan angka", line_number);
                            result.error = Some(format!("Invalid number: {}", fields[7]));
                            return result;
                        }
                    };
                    let ip_address = fields[8].trim();
                    let last_update = Utc::now().naive_utc();

                    let query = r#"
                        INSERT INTO TempImport (
                            Email, FullName, Age, Sex, Contact, ProductName,
                            ProductCount, Price, IPAddress, LastUpdate
                        )
                        VALUES (@P1, @P2, @P3, @P4, @P5, @P6, @P7, @P8, @P9, @P10);
                    "#;

                    match conn.execute(query, &[&email, &full_name, &age, &sex, &contact, &product_name, &product_count, &price, &ip_address, &last_update]).await {
                        Ok(res) => {
                            rowsaffected += res.rows_affected().iter().sum::<u64>();
                            send_ws_event("import_progress", &serde_json::json!({
                                "current": rowsaffected,
                                "total": total_count,
                                "message": result.message.clone()
                            }));
                        },
                        Err(e) => {
                            result.message = "Insert error".to_string();
                            result.error = Some(format!("Query failed: {}", e));
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
                "message": "Insert error",
                "error": result.error.clone()
            }));
            trans.rollback().await.ok();
            result.message = "Tidak ada data yang di-insert.".to_string();
        }

        result
    }

    pub async fn import_xlsx_from_file(file_path: PathBuf, connection: web::Data<Pool<ConnectionManager>>, has_header: bool) -> ActionResult<String, String> {
        let mut result = ActionResult::default();
        let mut rowsaffected: u64 = 0;

        let mut workbook = match open_workbook_auto(&file_path) {
            Ok(wb) => wb,
            Err(e) => {
                result.message = "Gagal membuka file Excel".to_string();
                result.error = Some(format!("Error: {}", e));
                return result;
            }
        };

        let range = match workbook.worksheet_range_at(0).ok_or("Sheet kosong") {
            Ok(Ok(r)) => r,
            _ => {
                result.message = "Worksheet tidak ditemukan atau error".into();
                result.error = Some("Sheet pertama tidak bisa diakses".into());
                return result;
            }
        };

        let total_count = range.height() as u64 - if has_header { 1 } else { 0 };

        let trans = match Transaction::begin(&connection).await {
            Ok(t) => t,
            Err(e) => {
                result.message = "Internal server error".to_string();
                result.error = Some(format!("Failed to begin transaction: {}", e));
                return result;
            }
        };

        for (i, row) in range.rows().enumerate() {
            if has_header && i == 0 {
                continue; // Skip header
            }

            if row.len() < 9 {
                result.message = format!("Baris {} tidak memiliki 10 kolom", i + 1);
                result.error = Some(format!("Ditemukan hanya {} kolom", row.len()));
                trans.rollback().await.ok();
                return result;
            }

            let extract = |i: usize| -> String {
                match row.get(i) {
                    Some(DataType::String(s)) => s.clone(),
                    Some(DataType::Float(f)) => f.to_string(),
                    Some(DataType::Int(n)) => n.to_string(),
                    Some(DataType::Bool(b)) => b.to_string(),
                    Some(DataType::Empty) | None => "".to_string(),
                    Some(val) => val.to_string(),
                }
            };

            let email        = extract(0);
            let full_name    = extract(1);
            let age: i32     = extract(2).parse().unwrap_or_default();
            let sex          = extract(3);
            let contact      = extract(4);
            let product_name = extract(5);
            let product_count: i32 = extract(6).parse().unwrap_or(0);
            let price: f64   = extract(7).parse().unwrap_or(0.0);
            let ip_address   = extract(8);
            let last_update  = Utc::now().naive_utc();

            let query = r#"
                INSERT INTO TempImport (
                    Email, FullName, Age, Sex, Contact, ProductName,
                    ProductCount, Price, IPAddress, LastUpdate
                ) VALUES (@P1, @P2, @P3, @P4, @P5, @P6, @P7, @P8, @P9, @P10);
            "#;

            if let Some(conn) = trans.conn.lock().await.as_mut() {
                match conn.execute(query, &[
                    &email, &full_name, &age, &sex, &contact, &product_name,
                    &product_count, &price, &ip_address, &last_update
                ]).await {
                    Ok(res) => {
                        rowsaffected += res.rows_affected().iter().sum::<u64>();

                        let progress = serde_json::json!({
                            "current": rowsaffected,
                            "total": total_count,
                            "message": result.message.clone()
                        });
                        send_ws_event("import_progress", &progress);
                    }
                    Err(e) => {
                        result.message = "Insert error".into();
                        result.error = Some(format!("Query failed: {}", e));
                    }
                }
            } else {
                result.message = "Failed to get connection".into();
                result.error = Some("DB connection error".into());
                return result;
            }
        }

        if rowsaffected > 0 {
            send_ws_event("import_done", &serde_json::json!({
                "result": true,
                "imported": rowsaffected,
                "message": "Import selesai"
            }));
            trans.commit().await.ok();
            result.result = true;
            result.message = format!("Berhasil import {} baris.", rowsaffected);
        } else {
            send_ws_event("import_error", serde_json::json!({
                "result": false,
                "imported": rowsaffected,
                "message": "Insert error",
                "error": result.error.clone()
            }));
            trans.rollback().await.ok();
            result.message = "Tidak ada data yang berhasil di-insert".into();
            result.error = Some("Semua baris gagal atau kosong".into());
        }

        result
    }
    
    pub async fn import_dbf_from_file(file_path: PathBuf, connection: web::Data<Pool<ConnectionManager>>) -> ActionResult<String, String> {
        let mut result = ActionResult::default();
        let mut rowsaffected: u64 = 0; 
        let trans = match Transaction::begin(&connection).await {
            Ok(trans) => trans,
            Err(e) => {
                result.message = "Failed to get connection".into();
                result.error = Some(e.to_string());
                return result;
            }
        };
        if let Err(e) = dbase::read(&file_path) {
            result.message = "Failed to read DBF".into();
            result.error = Some(e.to_string());
            return result;
        } else {
            let records = dbase::read(&file_path).unwrap();
            let total_count = records.len() as u64;

            match trans.conn.lock().await.as_mut() {
                Some(conn) => {
                    let query = r#"
                    INSERT INTO TempImport (
                        Email, FullName, Age, Sex, Contact, ProductName,
                        ProductCount, Price, IPAddress, LastUpdate
                    ) VALUES (@P1, @P2, @P3, @P4, @P5, @P6, @P7, @P8, @P9, @P10);
                    "#;

                    for record in records {
                        let email = match record.get("EMAIL") {
                            Some(FieldValue::Character(Some(s))) => s.clone(),
                            _ => String::new(),
                        };
                        let full_name = match record.get("FULLNAME") {
                            Some(FieldValue::Character(Some(s))) => s.clone(),
                            _ => String::new(),
                        };
                        let age: i32 = match record.get("AGE") {
                            Some(FieldValue::Numeric(Some(n))) => *n as i32,
                            _ => 0,
                        };
                        let sex = match record.get("SEX") {
                            Some(FieldValue::Character(Some(s))) => s.clone(),
                            _ => String::new(),
                        };
                        let contact = match record.get("CONTACT") {
                            Some(FieldValue::Character(Some(s))) => s.clone(),
                            _ => String::new(),
                        };
                        let product_name = match record.get("PRODUCTNAM") {
                            Some(FieldValue::Character(Some(s))) => s.clone(),
                            _ => String::new(),
                        };
                        let product_count: i32 = match record.get("PRODUCTCOU") {
                            Some(FieldValue::Numeric(Some(n))) => *n as i32,
                            _ => 0,
                        };
                        let price: f64 = match record.get("PRICE") {
                            Some(FieldValue::Numeric(Some(n))) => *n,
                            _ => 0.0,
                        };
                        let ip_address = match record.get("IPADDRESS") {
                            Some(FieldValue::Character(Some(s))) => s.clone(),
                            _ => String::new(),
                        };
                        let last_update = chrono::Utc::now();

                        match conn.execute(query, &[&email, &full_name, &age, &sex, &contact, &product_name, &product_count, &price, &ip_address, &last_update]).await {
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
                            }
                        }
                    }

                }
                None => {
                    result.message = "Failed to get connection".into();
                    result.error = Some("DB connection error".into());
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
                    "message": "Insert error",
                    "error": result.error.clone()
                }));
                trans.rollback().await.ok();
                result.message = "Tidak ada data yang di-insert.".to_string();
            }
        }

        return result;
    }

    pub async fn import_xml_from_file(file_path: PathBuf, connection: web::Data<Pool<ConnectionManager>>) -> ActionResult<String, String> {
        let mut result = ActionResult::default();
        let mut rowsaffected: u64 = 0;

        // Baca isi file XML
        let xml_content = match fs::read_to_string(&file_path) {
            Ok(s) => s,
            Err(e) => {
                result.message = "File open error".into();
                result.error = Some(format!("Failed to read XML: {}", e));
                return result;
            }
        };

        // Split berdasarkan <Record>
        let records: Vec<&str> = xml_content
            .split("<Record>")
            .skip(1) // skip header
            .map(|s| s.split("</Record>").next().unwrap_or("").trim())
            .collect();

        let total_count = records.len() as u64;

        let trans = match Transaction::begin(&connection).await {
            Ok(t) => t,
            Err(e) => {
                result.message = "Internal server error".into();
                result.error = Some(format!("Failed to begin transaction: {}", e));
                return result;
            }
        };

        if let Some(conn) = trans.conn.lock().await.as_mut() {
            for (idx, record) in records.iter().enumerate() {
                let get_tag = |field: &str| -> String {
                    record
                        .split(&format!("<{0}>", field))
                        .nth(1)
                        .and_then(|s| s.split(&format!("</{0}>", field)).next())
                        .unwrap_or("")
                        .trim()
                        .to_string()
                };

                // Ambil value masing-masing field dari string XML
                let email = get_tag("Email");
                let full_name = get_tag("FullName");
                let age = get_tag("Age").parse::<i32>().unwrap_or(0);
                let sex = get_tag("Sex");
                let contact = get_tag("Contact");
                let product_name = get_tag("ProductName");
                let product_count = get_tag("ProductCount").parse::<i32>().unwrap_or(0);
                let price = get_tag("Price").parse::<f64>().unwrap_or(0.0);
                let ip_address = get_tag("IPAddress");
                let last_update = Utc::now().naive_utc();

                // Jalankan insert
                let query = r#"
                    INSERT INTO TempImport (
                        Email, FullName, Age, Sex, Contact, ProductName,
                        ProductCount, Price, IPAddress, LastUpdate
                    )
                    VALUES (@P1, @P2, @P3, @P4, @P5, @P6, @P7, @P8, @P9, @P10);
                "#;

                match conn
                    .execute(
                        query,
                        &[
                            &email,
                            &full_name,
                            &age,
                            &sex,
                            &contact,
                            &product_name,
                            &product_count,
                            &price,
                            &ip_address,
                            &last_update,
                        ],
                    )
                    .await
                {
                    Ok(res) => {
                        rowsaffected += res.rows_affected().iter().sum::<u64>();
                        send_ws_event(
                            "import_progress",
                            &serde_json::json!({
                                "current": rowsaffected,
                                "total": total_count
                            }),
                        );
                    }
                    Err(e) => {
                        result.message = format!("Baris ke-{} gagal insert", idx + 1);
                        result.error = Some(format!("Query failed: {}", e));
                        return result;
                    }
                }
            }
        } else {
            result.message = "Internal server error".into();
            result.error = Some("Failed to get connection".into());
            return result;
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
                "message": "Insert error",
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

    pub async fn count_txt_lines<P: AsRef<Path>>(file_path: P, has_header: bool) -> std::io::Result<u64> {
        let file = File::open(file_path).await?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines();

        let mut count = 0u64;
        while lines.next_line().await?.is_some() {
            count += 1;
        }

        if !has_header && count > 0 {
            count -= 1;
        }

        Ok(count)
    }

    fn detect_delimiter(line: &str) -> Option<char> {
        let delimiters = [',', ';', '|'];
        let mut max_count = 0;
        let mut selected = None;

        for &d in &delimiters {
            let count = line.matches(d).count();
            if count > max_count {
                max_count = count;
                selected = Some(d);
            }
        }

        selected
    }
}