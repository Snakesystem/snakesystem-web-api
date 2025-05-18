use chrono::NaiveDateTime;
use serde_json::{json, Value};
use tiberius::{numeric::Numeric, ColumnType, Row};

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
}