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
    pub fn send_email(request: EmailRequest) -> ActionResult<String, String> {
        let mut result = ActionResult::default();

        let smtp_username = "8cf4d6002@smtp-brevo.com";
        let smtp_password = "m0bfcwQOYXkvr6qp";
        let smtp_server = "smtp-relay.brevo.com";

        // Baca template
        let template_str = include_str!("../../templates/mail_to.mustache");

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

        result
    }

    pub async  fn contact_form(req: HttpRequest, connection: web::Data<Pool<ConnectionManager>>, request: EmailRequest) -> ActionResult<(), String> {

        let mut result: ActionResult<(), String> = ActionResult::default();

        match connection.clone().get().await {
            Ok(mut conn) => {
                let query_result: Result<QueryStream, _> = conn.query(
                    r#"SELECT Name, Receiver, Subject, Message, SentCount, IsEnabled, IPAddress, LastUpdate 
                    FROM EmailHistory 
                    WHERE Receiver = @P1"#, &[&request.recipient]).await;
                match query_result {
                    Ok(rows) => {
                        if let Ok(Some(row)) = rows.into_row().await {
                            match Transaction::begin(&connection).await {
                                Ok(trans) => {
                                    let sent_count: i32 = row.get("SentCount").unwrap_or(0);
                                    let isenabled: bool = row.get::<bool, _>("IsEnabled").unwrap_or(false);
                                    let last_update: NaiveDateTime = row.get::<NaiveDateTime, _>("LastUpdate").unwrap_or(Utc::now().naive_utc());
                                    let updated_count: i32 = sent_count + 1;
                                    let enabled: bool = updated_count != 2;

                                    if sent_count < 2 && isenabled {
                                        match trans.conn.lock().await.as_mut() {
                                            Some(conn) => {
                                                
                                                if let Err(err) = conn.execute(
                                                    r#"UPDATE [dbo].[EmailHistory]
                                                        set [SentCount] = @P2, [Subject] = @P3, [Message] = @P4, [IPAddress] = @P5, [IsEnabled] = @P6, [LastUpdate] = @P7
                                                        WHERE Receiver = @P1"#,
                                                    &[
                                                        &request.recipient, 
                                                        &updated_count, 
                                                        &request.subject, 
                                                        &request.message, &GenericService::get_ip_address(&req),
                                                        &enabled,
                                                        &Utc::now().naive_utc()
                                                    ],
                                                ).await {
                                                    result.error = Some(format!("Fauled: {:?}", err));
                                                    return result;
                                                }
                                            }
                                            None => {
                                                result.error = Some("Failed to get database connection".into());
                                                return result;
                                            }
                                        }
                                    } else if Utc::now().naive_utc().signed_duration_since(last_update) > chrono::Duration::hours(24) {
                                        match trans.conn.lock().await.as_mut() {
                                            Some(conn) => {
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
                                                    result.error = Some(format!("Fauled: {:?}", err));
                                                    return result;
                                                }
                                            }
                                            None => {
                                                result.error = Some("Failed to get database connection".into());
                                                return result;
                                            }
                                        }
                                    } else {
                                        result.error = Some("You are only allowed to send 2 emails a day".into());
                                        result.message = "Send limit exceeded".to_string();
                                        return result;
                                    }
                                    // 🔵 Commit transaksi
                                    if let Err(err) = trans.commit().await {
                                        result.error = Some(format!("Failed to commit transaction: {:?}", err));
                                        return result;
                                    }
                    
                                    result.result = true;
                                    result.message = "Change password successfully".to_string();
                                }
                                Err(err) => {
                                    result.error = Some(format!("Failed to start transaction: {:?}", err));
                                }
                            }
                    
                        } else {
                            match Transaction::begin(&connection).await {
                                Ok(trans) => {
                                    // 🔴 Scope ketiga: Insert ke TableRequest
                                    match trans.conn.lock().await.as_mut() {
                                            Some(conn) => {
                                                if let Err(err) = conn.execute(
                                                    r#"INSERT INTO [dbo].[EmailHistory] 
                                                    ([Name], [Receiver], [Subject], [Message], [SentCount], [IsEnabled], [IPAddress]) 
                                                    VALUES (@P1, @P2, @P3, @P4, @P5, @P6, @P7)"#,
                                                    &[
                                                        &request.name, 
                                                        &request.recipient, 
                                                        &request.subject, 
                                                        &request.message, 
                                                        &1, 
                                                        &false, 
                                                        &GenericService::get_ip_address(&req)
                                                    ],
                                                ).await {
                                                    result.error = Some(format!("Fauled: {:?}", err));
                                                    return result;
                                                }
                                            }
                                            None => {
                                                result.error = Some("Failed to get database connection".into());
                                                return result;
                                            }
                                        }
                    
                                    // 🔵 Commit transaksi
                                    if let Err(err) = trans.commit().await {
                                        result.error = Some(format!("Failed to commit transaction: {:?}", err));
                                        return result;
                                    }
                    
                                    result.result = true;
                                    result.message = "Change password successfully".to_string();
                                }
                                Err(err) => {
                                    result.error = Some(format!("Failed to start transaction: {:?}", err));
                                }
                            }
                            result.message = format!("No user found for email");
                            return result;
                        }
                    },
                    Err(err) => {
                        result.error = format!("Query execution failed: {:?}", err).into();
                        return result;
                    },
                }
            },
            Err(err) => {
                result.error = format!("Internal Server error: {:?}", err).into();
                return result;
            }, 
        }

        return result;
        
    }
}