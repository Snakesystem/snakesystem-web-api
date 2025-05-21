// export_service.rs
use std::{fs::File, io::BufWriter, path::Path};
use actix_web::web;
use bb8::Pool;
use bb8_tiberius::ConnectionManager;
use tiberius::Row;
use chrono::{NaiveDateTime, Datelike};
use dbase::{TableWriterBuilder, Record, FieldName, FieldType, FieldValue};

pub struct ExportService;

impl ExportService {
    pub async fn export_db_to_dbf<P: AsRef<Path>>(
        connection: web::Data<Pool<ConnectionManager>>,
        output_path: P,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let query = r#"
            SELECT Email, FullName, Age, Sex, Contact,
                   ProductName, ProductCount, Price,
                   IPAddress, LastUpdate
            FROM TempImport
        "#;

        let stream = connection.get().await?.simple_query(query).await?;
        let mut results = stream.into_row_stream();

        // open the file & builder
        let file = File::create(output_path)?;
        let writer = BufWriter::new(file);
        let fields = vec![
            (FieldName::try_from("Email")?,       FieldType::Character(Some(100))),
            (FieldName::try_from("FullName")?,    FieldType::Character(Some(100))),
            (FieldName::try_from("Age")?,         FieldType::Numeric { size: 3,  decimals: 0 }),
            (FieldName::try_from("Sex")?,         FieldType::Character(Some(10))),
            (FieldName::try_from("Contact")?,     FieldType::Character(Some(50))),
            (FieldName::try_from("ProductName")?, FieldType::Character(Some(100))),
            (FieldName::try_from("ProductCount")?,FieldType::Numeric { size: 5,  decimals: 0 }),
            (FieldName::try_from("Price")?,       FieldType::Numeric { size: 10, decimals: 2 }),
            (FieldName::try_from("IPAddress")?,   FieldType::Character(Some(50))),
            (FieldName::try_from("LastUpdate")?,  FieldType::Date),
        ];
        let mut table = TableWriterBuilder::new()
            .add_fields(&fields)?
            .build(writer)?;

        // write each row
        while let Some(row) = results.try_next().await? {
            let record = Self::convert_row_to_record(&row)?;
            table.write_record(&record)?;
        }

        table.flush()?;
        Ok(())
    }

    fn convert_row_to_record(row: &Row) -> Result<Record, Box<dyn std::error::Error>> {
        let mut record = Record::default();

        // pull each column out (unwrap_or defaults)
        let email         : &str               = row.get("Email").unwrap_or("");
        let fullname      : &str               = row.get("FullName").unwrap_or("");
        let age           : i32                = row.get("Age").unwrap_or(0);
        let sex           : &str               = row.get("Sex").unwrap_or("");
        let contact       : &str               = row.get("Contact").unwrap_or("");
        let product_name  : &str               = row.get("ProductName").unwrap_or("");
        let product_count : i32                = row.get("ProductCount").unwrap_or(0);
        let price         : f64                = row.get("Price").unwrap_or(0.0);
        let ip_address    : &str               = row.get("IPAddress").unwrap_or("");
        let last_update   : Option<NaiveDateTime> = row.get("LastUpdate");

        // insert into DBF record
        record.insert("Email".try_into()?,       FieldValue::Character(Some(email.to_string())));
        record.insert("FullName".try_into()?,    FieldValue::Character(Some(fullname.to_string())));
        record.insert("Age".try_into()?,         FieldValue::Numeric(age.into()));
        record.insert("Sex".try_into()?,         FieldValue::Character(Some(sex.to_string())));
        record.insert("Contact".try_into()?,     FieldValue::Character(contact.to_string()));
        record.insert("ProductName".try_into()?, FieldValue::Character(product_name.to_string()));
        record.insert("ProductCount".try_into()?,
                      FieldValue::Numeric(product_count.into()));
        record.insert("Price".try_into()?,       FieldValue::Numeric(price));
        record.insert("IPAddress".try_into()?,
                      FieldValue::Character(ip_address.to_string()));

        // map NaiveDateTime -> NaiveDate, defaulting if none
        let date_value = last_update.map(|dt| {
            chrono::NaiveDate::from_ymd_opt(dt.year(), dt.month(), dt.day())
                .unwrap_or_else(|| chrono::NaiveDate::from_ymd(1970, 1, 1))
        });
        record.insert("LastUpdate".try_into()?, FieldValue::Date(date_value));

        Ok(record)
    }
}
