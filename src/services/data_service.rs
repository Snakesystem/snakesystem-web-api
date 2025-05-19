use std::collections::HashMap;
use std::fmt::Write;
use actix_web::web;
use bb8::Pool;
use bb8_tiberius::ConnectionManager;
use chrono::{NaiveDate, NaiveDateTime};
use serde_json::{json, Value};
use tiberius::{numeric::Numeric, ColumnType, Row};

use crate::contexts::model::{ActionResult, QueryClass, ResultList, TableDataParams};

pub struct DataService;

impl DataService {
    pub fn row_to_json(row: &Row) -> Value {
        let mut json_obj = serde_json::Map::new();

        for (i, col) in row.columns().iter().enumerate() {
            let col_name = col.name();
            let column_type = &col.column_type();

            match column_type {
                ColumnType::NVarchar | ColumnType::BigVarChar | ColumnType::Text => {
                    if let Ok(Some(value)) = row.try_get::<&str, _>(i) {
                        json_obj.insert(col_name.to_string(), json!(value));
                    } else {
                        json_obj.insert(col_name.to_string(), json!(null));
                    }
                },
                ColumnType::Int4 | ColumnType::Int8 | ColumnType::Intn => {
                    if let Ok(value) = row.try_get::<i32, _>(i) {
                        json_obj.insert(col_name.to_string(), json!(value));
                    } else {
                        json_obj.insert(col_name.to_string(), json!(null));
                    }
                },
                ColumnType::Bit => {
                    if let Ok(value) = row.try_get::<bool, _>(i) {
                        json_obj.insert(col_name.to_string(), json!(value));
                    } else {
                        json_obj.insert(col_name.to_string(), json!(null));
                    }
                },
                ColumnType::Datetimen => {
                    if let Ok(value) = row.try_get::<NaiveDateTime, _>(i) {
                        json_obj.insert(col_name.to_string(), json!(value));
                    } else {
                        json_obj.insert(col_name.to_string(), json!(null));
                    }
                },
                ColumnType::BigBinary => {
                    if let Ok(value) = row.try_get::<&[u8], _>(i) {
                        json_obj.insert(col_name.to_string(), json!(value));
                    } else {
                        json_obj.insert(col_name.to_string(), json!(null));
                    }
                },
                ColumnType::Numericn => {
                    if let Ok(Some(numeric)) = row.try_get::<Numeric, _>(i) {
                        let raw_value = numeric.value();
                        let scale = numeric.scale();
                        let divisor = 10i128.pow(scale as u32);
                        let int_part = raw_value / divisor;
                        let frac_raw = raw_value.abs() % divisor;

                        // Tentukan presisi berdasarkan scale
                        let frac_str = format!("{:0>width$}", frac_raw, width = scale as usize);

                        // Gabungkan bagian integer dan fraction (desimal)
                        let formatted = format!("{}.{}", int_part, frac_str);
                        
                        // Jika ingin menyimpan sebagai float dengan presisi, kita convert ke f64
                        let float_value = formatted.parse::<f64>().unwrap_or(0.0);  // Parsing string ke f64

                        json_obj.insert(col_name.to_string(), json!(float_value));
                    } else {
                        json_obj.insert(col_name.to_string(), json!(null));
                    }
                },
                _ => {
                    json_obj.insert(col_name.to_string(), json!(null));
                }
            }
        }

        Value::Object(json_obj)
    }

    pub async fn get_header(connection: web::Data<Pool<ConnectionManager>>, tablename: String) -> ActionResult<Vec<serde_json::Value>, String> {
        let mut result: ActionResult<Vec<serde_json::Value>, String> = ActionResult::default();
    
        match connection.clone().get().await {
            Ok(mut conn) => {
                let sql = r#"
                    DECLARE @Result NVARCHAR(MAX);
                    EXEC dbo.Web_CreateTableObject @ViewName = @P1, @Result = @Result OUTPUT;
                    SELECT @Result AS Result;
                "#;
    
                match conn.query(sql, &[&tablename]).await {
                    Ok(rows) => {
                        let all_results = rows.into_results().await.unwrap();
    
                        // Ambil isi @Result dari SELECT terakhir
                        let output_result = all_results.last()
                            .and_then(|set| set.first())
                            .and_then(|row| row.get::<&str, _>("Result"))
                            .unwrap_or("");
    
                        // Parse isi @Result ke Vec<serde_json::Value>
                        match serde_json::from_str::<Vec<serde_json::Value>>(output_result) {
                            Ok(parsed_json) => {
                                result.data = Some(parsed_json);
                                result.result = true;
                                result.message = "Data retrieved successfully".to_string();
                            }
                            Err(e) => {
                                result.message = "Failed to parse JSON".to_string();
                                result.error = Some(e.to_string());
                            }
                        }
    
                        return result;
                    }
                    Err(e) => {
                        result.message = "Query failed".to_string();
                        result.error = Some(e.to_string());
                        return result;
                    }
                }
            }
            Err(e) => {
                result.message = "Connection failed".to_string();
                result.error = Some(e.to_string());
                return result;
            }
        }
    }

    pub async fn get_table_data(allparams: TableDataParams, connection: web::Data<Pool<ConnectionManager>>) -> Result<ResultList, Box<dyn std::error::Error>> {
        let mut result = ResultList {
            totalNotFiltered: 0,
            total: 0,
            rows: vec![],
        };
    
        let query = Self::get_query_table(allparams.clone(), false);
    
        let mut client = connection.get().await?;
    
        if !allparams.tablename.is_empty() {
            let row: Option<Row> = client.query(query.query_total_all.clone(), &[]).await?.into_row().await?;
            if let Some(r) = row {
                result.totalNotFiltered = r.try_get::<i32, _>(0)?.unwrap_or(0);
            }
        }
    
        // Hitung total data yang sesuai filter
        if let Some(filter) = &allparams.filter {
            if filter != "{filter:undefined}" {
                let row: Option<Row> = client.query(query.query_total_with_filter.clone(), &[]).await?.into_row().await?;
                if let Some(r) = row {
                    result.total = r.try_get::<i32, _>(0)?.unwrap_or(0);
                }
            }
        } else {
            result.total = result.totalNotFiltered;
        }
    
        let rows = client.query(query.query.clone(), &[]).await?.into_results().await?;
        result.rows = rows.into_iter()
            .flat_map(|r| r.into_iter())
            .map(|row| Self::row_to_json(&row))  // 🔥 Ubah `Row` ke JSON
            .collect();
    
        Ok(result)
    }
    
    fn get_query_table(allparams: TableDataParams, bypass_skip: bool) -> QueryClass {
        let mut result = QueryClass {
            query: String::new(),
            query_total_all: String::new(),
            query_total_with_filter: String::new(),
        };
    
        if allparams.limit == 0 {
            return result;
        }
    
        let tablename = format!("[{}]", allparams.tablename);
        let mut query_total_all = format!("SELECT count(*) as total FROM {}", tablename);
        let mut q_and_where = String::from(" WHERE 1=1 ");
        let mut q_order_by = String::new();
        let mut q_skip_row = String::new();
        let mut q_and_where_for_total_with_filter = String::from(" WHERE 1=1 ");
    
        // Gunakan `nidkey` sebagai primary key jika tersedia
        let q_primary_key = allparams.nidkey.clone().unwrap_or_else(|| "AutoNID".to_string());
        let q_primary_key_order = q_primary_key.clone();
    
        // Tambahkan filter jika ada
        if let Some(filter) = &allparams.filter {
            if filter != "{filter:undefined}" {
                if let Ok(filter_name) = serde_json::from_str::<HashMap<String, String>>(filter) {
                    if !filter_name.is_empty() {
                        let q_and_where_result = Self::get_query_table_where(q_and_where.clone(), filter_name);
    
                        q_and_where = q_and_where_result.clone();
                        q_and_where_for_total_with_filter = q_and_where_result.clone();
                    }
                }
            }
        }
    
        query_total_all.push_str(&q_and_where);
    
        let query_total_with_filter = format!(
            "SELECT count(*) as totalWithFilter FROM {} {}",
            tablename, q_and_where_for_total_with_filter
        );
    
        result.query_total_with_filter = query_total_with_filter;
    
        // Sorting & pagination
        if !bypass_skip {
            if let Some(sort) = &allparams.sort {
                if let Some(order) = &allparams.order {
                    let _ = write!(q_order_by, " ORDER BY {} {}", sort, order);
                }
            } else {
                let _ = write!(q_order_by, " ORDER BY {} DESC", q_primary_key_order);
            }
    
            let _ = write!(
                q_skip_row,
                " OFFSET {} ROWS FETCH NEXT {} ROWS ONLY",
                allparams.offset, allparams.limit
            );
        }
    
        // Query utama
        result.query = format!(
            "SELECT * FROM {} {} {} {}",
            tablename, q_and_where, q_order_by, q_skip_row
        );
    
        result.query_total_all = query_total_all;
        result
    }

    pub fn get_query_table_where(mut fquery: String, filter_name: HashMap<String, String>) -> String {
        for (key, value) in filter_name {
            if let Ok(temp_date) = NaiveDate::parse_from_str(&value, "%Y-%m-%d") {
                if key.ends_with("Date") {
                    let next_date = temp_date.succ_opt().unwrap_or(temp_date);
                    let _ = write!(
                        fquery,
                        " AND {} BETWEEN '{}' AND '{}'",
                        key, value, next_date
                    );
                } else {
                    let _ = write!(fquery, " AND {} = '{}'", key, value);
                }
            } else if key.ends_with("Time") {
                let dates: Vec<&str> = value.split("to").collect();
                if dates.len() == 2 {
                    let _ = write!(
                        fquery,
                        " AND {} BETWEEN '{} 00:00:00' AND '{} 23:59:59'",
                        key, dates[0], dates[1]
                    );
                }
            } else if key.starts_with('_') || key.ends_with("NID") || key.ends_with("ID") {
                let _ = write!(fquery, " AND {} = '{}'", key, value);
            } else {
                let _ = write!(fquery, " AND {} LIKE '%{}%'", key, value);
            }
        }
    
        fquery
    }
}