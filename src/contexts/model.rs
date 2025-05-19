use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use validator::Validate;
use crate::utils::validation::validator::{
    required, valid_phone_number, valid_name, required_int, valid_password
}; 

#[derive(Debug, Serialize, ToSchema)]
pub struct ActionResult<T, E> {
    pub result: bool,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<E>,
}

// Implementasi Default
impl<T, E> Default for ActionResult<T, E> {
    fn default() -> Self {
        Self {
            result: false, // Default-nya false
            message: String::new(),
            data: None,
            error: None,
        }
    }
}

fn serialize_datetime<S>(dt: &DateTime<Utc>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let formatted = dt.format("%Y-%m-%d %H:%M:%S").to_string();
    serializer.serialize_str(&formatted)
}

#[derive(Debug, Serialize, Clone)]
pub struct Company {
    pub company_id: String,
    pub company_name: String,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct LoginRequest {
    #[validate(required, email(message = "Invalid email format"))]
    pub email: Option<String>,

    #[validate(custom(function = "required"), custom(function = "valid_password"))]
    pub password: Option<String>,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct RegisterRequest {
    #[validate(required, email(message = "Invalid email format"))]
    pub email: Option<String>,

    #[validate(custom(function = "required"), custom(function = "valid_password"))]
    pub password: Option<String>,

    #[validate(custom(function = "required"), custom(function = "valid_phone_number"))]
    pub mobile_phone: Option<String>,

    #[validate(custom(function = "required"), custom(function = "valid_name"))]
    pub full_name: Option<String>,

    #[serde(default)]
    pub sales: i32,
    
    #[serde(default)]
    pub referal: String,

    #[validate(custom(function = "required_int"))]
    pub client_category: Option<i32>,

    #[serde(default)]
    pub app_ipaddress: String
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct ResetPasswordRequest {
    #[validate(required, email(message = "Invalid email format"))]
    pub email: Option<String>,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct ChangePasswordRequest {
    #[validate(required, email(message = "Invalid email format"))]
    pub email: Option<String>,

    #[validate(custom(function = "required"), custom(function = "valid_password"))]
    pub password: Option<String>,

    pub reset_password_key: String
}

#[derive(Debug, Serialize, Clone)]
pub struct WebUser {
    pub auth_usernid: i32,
    pub email: String,
    pub mobile_phone: String,
    pub disabled_login: bool,
    pub picture: Option<String>,
    #[serde(serialize_with = "serialize_datetime")]
    pub register_date: chrono::DateTime<Utc>
}

#[derive(Debug,Deserialize, Serialize)]
pub struct ContactRequest {
    pub name: String,
    pub email: String,
    pub subject: String,
    pub message: String,
}

#[derive(Debug, Deserialize,Serialize, ToSchema, Clone, Validate)]
pub struct EmailRequest {
    #[validate(custom(function = "required"))]
    pub name: String,
    #[validate(custom(function = "required"))]
    pub subject: String,
    #[validate(custom(function = "required"))]
    pub recipient: String,
    #[validate(custom(function = "required"))]
    pub message: String,
}

// Region Library
#[derive(Debug, Deserialize, Serialize, ToSchema, Clone, Validate)]
pub struct NewNoteRequest {
    #[validate(custom(function = "required"))]
    pub category: String,
    #[validate(custom(function = "required"))]
    pub title: String,
    pub slug: Option<String>,
    #[validate(custom(function = "required"))]
    pub content_md: String,
}

#[derive(Debug, Deserialize, Serialize, ToSchema, Clone)]
pub struct Notes {
    pub note_id: i32,
    pub category: String,
    pub title: String,
    pub slug: String,
    pub content_md: String,
    pub ip_address: String,
    #[serde(serialize_with = "serialize_datetime")]
    pub last_update: chrono::DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize, IntoParams)]
pub struct HeaderParams {
    pub tablename: String,
}

#[derive(Debug, Clone, Deserialize, IntoParams)]
pub struct TableDataParams {
    pub tablename: String,
    pub limit: i32,
    pub offset: i32,
    #[param(required = false)]
    pub filter: Option<String>,
    pub sort: Option<String>,
    pub order: Option<String>,
    pub nidkey: Option<String>,
    // pub nidvalue: Option<String>,
}

#[derive(Debug)]
pub struct QueryClass {
    pub query: String,
    pub query_total_all: String,
    pub query_total_with_filter: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ResultList {
    pub totalNotFiltered: i32,
    pub total: i32,
    pub rows: Vec<serde_json::Value>, // Pastikan ini bisa dikonversi ke JSON
}

#[derive(Debug, Deserialize)]
pub struct MyRow {
    pub id: i32,
    pub name: String,
    // tambah field lain sesuai kebutuhan
}
