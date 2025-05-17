use std::{collections::HashMap};
use actix_web::{web, HttpRequest};
use bb8::Pool;
use bb8_tiberius::ConnectionManager;
use chrono::{NaiveDateTime, Utc};
use handlebars::Handlebars;
use lettre::{transport::smtp::authentication::Credentials, Message, SmtpTransport, Transport};
use tiberius::QueryStream;

use crate::contexts::{connection::Transaction, model::{ActionResult, EmailRequest}};

use super::generic_service::GenericService;

pub struct MailService;

impl MailService {
    pub async  fn send_email_to(request: EmailRequest) -> ActionResult<String, String> {
        let mut result: ActionResult<String, String> = ActionResult::default();

        let smtp_username = "8cf4d6002@smtp-brevo.com";
        let smtp_password = "m0bfcwQOYXkvr6qp";
        let smtp_server = "smtp-relay.brevo.com";

        // Baca template
        let template_str = include_str!("../../templates/mail_to.mustache");
        let res_svg = reqwest::get("https://snakesystem.github.io/svg/email_send.svg").await.unwrap();
        if res_svg.status().is_success() {
            println!("Succes load image email");
        }

        let res_fb = reqwest::get("https://cdn-icons-png.flaticon.com/512/733/733547.png").await.unwrap();
        if res_fb.status().is_success() {
            println!("Succes load image facebook");
        }

        let res_linkedin = reqwest::get("https://cdn-icons-png.flaticon.com/512/733/733561.png").await.unwrap();
        if res_linkedin.status().is_success() {
            println!("Succes load image linkedin");
        }

        let res_ig = reqwest::get("https://cdn-icons-png.flaticon.com/512/733/733558.png").await.unwrap();
        if res_ig.status().is_success() {
            println!("Succes load image instagram");
        }

        let res_ss = reqwest::get("https://snakesystem.github.io/favicon.ico").await.unwrap();
        if res_ss.status().is_success() {
            println!("Succes load image favicon");
        }

        // Setup handlebars
        let mut handlebars = Handlebars::new();
        handlebars.register_template_string("mail_to", template_str).unwrap();

        // Data untuk templating
        let mut data = HashMap::new();
        data.insert("subject", request.subject.as_str());
        data.insert("message", &request.message.as_str());
        data.insert("name", &&request.name.as_str());
        data.insert("recipient", &&&request.recipient.as_str());

        let html_body = handlebars.render("mail_to", &data).unwrap();

        let email = Message::builder()
            .from("techsnakesystem@gmail.com".parse().unwrap())
            .to(request.recipient.parse().unwrap())
            .subject(&request.subject)
            .header(lettre::message::header::ContentType::TEXT_HTML)
            .body(html_body)
            .unwrap();

        let creds = Credentials::new(smtp_username.to_string(), smtp_password.to_string());

        let mailer = SmtpTransport::relay(smtp_server)
            .unwrap()
            .credentials(creds)
            .build();

        match mailer.send(&email) {
            Ok(res) => {
                println!("Email sent: {:#?}", res);
                result.result = true;
                result.message = "Email sent successfully!".to_string();
            }
            Err(e) => {
                eprintln!("Failed to send email: {e}");
                result.result = false;
                result.error = Some(e.to_string());
            }
        }

        return result;
    }

    pub async  fn send_email_from(request: EmailRequest) -> ActionResult<String, String> {
        let mut result: ActionResult<String, String> = ActionResult::default();

        let smtp_username = "8cf4d6002@smtp-brevo.com";
        let smtp_password = "m0bfcwQOYXkvr6qp";
        let smtp_server = "smtp-relay.brevo.com";

        // Baca template
        let template_str = include_str!("../../templates/mail_from.mustache");
        let res_svg = reqwest::get("https://snakesystem.github.io/svg/email_send.svg").await.unwrap();
        if res_svg.status().is_success() {
            println!("Succes load image email");
        }

        let res_fb = reqwest::get("https://cdn-icons-png.flaticon.com/512/733/733547.png").await.unwrap();
        if res_fb.status().is_success() {
            println!("Succes load image facebook");
        }

        let res_linkedin = reqwest::get("https://cdn-icons-png.flaticon.com/512/733/733561.png").await.unwrap();
        if res_linkedin.status().is_success() {
            println!("Succes load image linkedin");
        }

        let res_ig = reqwest::get("https://cdn-icons-png.flaticon.com/512/733/733558.png").await.unwrap();
        if res_ig.status().is_success() {
            println!("Succes load image instagram");
        }

        let res_ss = reqwest::get("https://snakesystem.github.io/favicon.ico").await.unwrap();
        if res_ss.status().is_success() {
            println!("Succes load image favicon");
        }

        // Setup handlebars
        let mut handlebars = Handlebars::new();
        handlebars.register_template_string("mail_from", template_str).unwrap();

        // Data untuk templating
        let mut data = HashMap::new();
        data.insert("subject", request.subject.as_str());
        data.insert("message", &request.message.as_str());
        data.insert("name", &&request.name.as_str());
        data.insert("recipient", &&&request.recipient.as_str());

        let html_body = handlebars.render("mail_from", &data).unwrap();

        let email = Message::builder()
            .from("techsnakesystem@gmail.com".parse().unwrap())
            .to(request.recipient.parse().unwrap())
            .subject(&request.subject)
            .header(lettre::message::header::ContentType::TEXT_HTML)
            .body(html_body)
            .unwrap();

        let creds = Credentials::new(smtp_username.to_string(), smtp_password.to_string());

        let mailer = SmtpTransport::relay(smtp_server)
            .unwrap()
            .credentials(creds)
            .build();

        match mailer.send(&email) {
            Ok(res) => {
                println!("Email sent: {:#?}", res);
                result.result = true;
                result.message = "Email sent successfully!".to_string();
            }
            Err(e) => {
                eprintln!("Failed to send email: {e}");
                result.result = false;
                result.error = Some(e.to_string());
            }
        }

        return result;
    }

    pub async  fn contact_form(req: HttpRequest, connection: web::Data<Pool<ConnectionManager>>, request: EmailRequest) -> ActionResult<(), String> {

        let mut result: ActionResult<(), String> = ActionResult::default();

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
                let query_result: Result<QueryStream, _> = conn.query(
                    r#"SELECT Name, Receiver, Subject, Message, SentCount, IsEnabled, IPAddress, LastUpdate 
                    FROM EmailHistory 
                    WHERE Receiver = @P1"#, &[&request.recipient]).await;
                if let Err(err) = query_result {
                    result.message = "Internal server error".to_string();
                    result.error = Some(format!("Query error: {}", err));
                    return result;
                } else {
                    println!("Select data");
                    let rows = query_result.unwrap();
                    if let Ok(Some(row)) = rows.into_row().await {
                        let sent_count: i32 = row.get("SentCount").unwrap_or(0);
                        let isenabled: bool = row.get::<bool, _>("IsEnabled").unwrap_or(false);
                        let last_update: NaiveDateTime = row.get::<NaiveDateTime, _>("LastUpdate").unwrap_or(Utc::now().naive_utc());
                        let updated_count: i32 = sent_count + 1;
                        let enabled: bool = updated_count != 2;

                        if sent_count < 2 && isenabled {
                            if let Err(err) = conn.execute(
                                r#"UPDATE [dbo].[EmailHistory]
                                    set [SentCount] = @P2, [Subject] = @P3, [Message] = @P4, [IPAddress] = @P5, [IsEnabled] = @P6, [LastUpdate] = @P7
                                    WHERE Receiver = @P1"#,
                                &[&request.recipient, &updated_count, &request.subject, &request.message, &GenericService::get_ip_address(&req), &enabled, &last_update]
                            ).await {
                                result.message = "Internal server error".to_string();
                                result.error = Some(format!("Query error: {}", err));
                                return result;
                            }
                            result.result = true;
                            result.message = "Email sent successfully!".to_string();
                        } else if Utc::now().naive_utc().signed_duration_since(last_update) > chrono::Duration::hours(24) {
                            if let Err(err) = conn.execute(
                                r#"UPDATE [dbo].[EmailHistory]
                                    set [SentCount] = @P2, [Subject] = @P3, [Message] = @P4, [IPAddress] = @P5, [IsEnabled] = @P6, [LastUpdate] = @P7
                                    WHERE Receiver = @P1"#,
                                &[
                                    &request.recipient, 
                                    &1, 
                                    &request.subject, 
                                    &request.message, &GenericService::get_ip_address(&req),
                                    &true,
                                    &Utc::now().naive_utc()
                                ],
                            ).await {
                                result.message = "Internal server error".to_string();
                                result.error = Some(format!("Failed: {:?}", err));
                                return result;
                            }
                            result.result = true;
                            result.message = "Email sent successfully!".to_string();
                        } else {
                            result.message = "Has exceeded the limit".to_string();
                            result.error = Some("You can only send emails twice per day.".to_string());
                        }
                    } else {
                        println!("Email masul 4");
                        if let Err(err) = conn.execute(
                            r#"INSERT INTO [dbo].[EmailHistory] ([Name], [Receiver], [Subject], [Message], [SentCount], [IsEnabled], [IPAddress], [LastUpdate])
                            VALUES (@P1, @P2, @P3, @P4, @P5, @P6, @P7, @P8)"#,
                            &[
                                &request.name,
                                &request.recipient,
                                &request.subject,
                                &request.message,
                                &1,
                                &true,
                                &GenericService::get_ip_address(&req),
                                &Utc::now().naive_utc(),
                            ],
                        ).await {
                            result.message = "Internal server error".to_string();
                            result.error = Some(format!("Failed: {}", err));
                            return result;
                        }

                        result.result = true;
                        result.message = "Email sent successfully!".to_string();
                    }
                }
            }
            None => {
                result.message = "Internal server error".to_string();
                result.error = Some("Failed to get connection".to_string());
                return result;
            }
        }

        // 🔵 Commit transaksi
        if let Err(err) = trans.commit().await {
            result.result = false;
            result.message = "Internal server error".to_string();
            result.error = Some(format!("Failed to commit transaction: {}", err));
            return result;
        }

        return result;
        
    }
}