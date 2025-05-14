use lettre::{transport::smtp::authentication::Credentials, Message, SmtpTransport, Transport};

use crate::contexts::model::{ActionResult, EmailRequest};

pub struct MailService;

impl MailService {
    pub fn send_email(request: EmailRequest) -> ActionResult<String, String> {

        let mut result = ActionResult::default();

        let smtp_username = "8cf4d6002@smtp-brevo.com"; // Login dari Brevo
        let smtp_password = "m0bfcwQOYXkvr6qp";           // Ambil dari SMTP Brevo
        let smtp_server = "smtp-relay.brevo.com";

        let email = Message::builder()
        .from("techsnakesystem@gmail.com".parse().unwrap())
        .to(request.recipient.parse().unwrap())
        .subject(&request.subject)
        .body(String::from("<h1>Hello</h1><p>This is a test from Rust!</p>"))
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
            },
            Err(e) => {
                eprintln!("Failed to send email: {e}");
                result.result = false;
                result.error = Some(e.to_string());
            }
        }

        return result
    }
}