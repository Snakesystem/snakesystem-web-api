use std::{collections::HashMap, fs};

use handlebars::Handlebars;
use lettre::{transport::smtp::authentication::Credentials, Message, SmtpTransport, Transport};

use crate::contexts::model::{ActionResult, EmailRequest};

pub struct MailService;

impl MailService {
    pub fn send_email(request: EmailRequest) -> ActionResult<String, String> {
        let mut result = ActionResult::default();

        let smtp_username = "8cf4d6002@smtp-brevo.com";
        let smtp_password = "m0bfcwQOYXkvr6qp";
        let smtp_server = "smtp-relay.brevo.com";

        // Baca template
        let template_str = fs::read_to_string("templates/mail_to.mustache")
            .expect("Template mail.handlebars tidak bisa dibaca");

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
}