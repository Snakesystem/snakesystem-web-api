use actix_web::{post, web, HttpResponse, Responder, Scope};
use lettre::{transport::smtp::authentication::Credentials, Message, SmtpTransport, Transport as _};

use crate::{contexts::model::{ContactRequest, EmailRequest}, services::mail_service::MailService};

pub fn mail_scope() -> Scope {
    
    web::scope("/email")
        .service(contact_form)
        .service(send_campaign)
}

#[post("/contact")]
async fn contact_form(form: web::Json<EmailRequest>) -> impl Responder {
    let contact = form.into_inner();
    
    let result = MailService::send_email(contact);

    if result.result {
        HttpResponse::Ok().json(result)
    } else {
        HttpResponse::InternalServerError().json(result)
    }
}

#[post("/send-campaign")]
async fn send_campaign(req: web::Json<ContactRequest>) -> impl Responder {
    let smtp_username = "8cf4d6002@smtp-brevo.com"; // Login dari Brevo
    let smtp_password = "m0bfcwQOYXkvr6qp";           // Ambil dari SMTP Brevo
    let smtp_server = "smtp-relay.brevo.com";

    let email = Message::builder()
    .from("techsnakesystem@gmail.com".parse().unwrap())
    .reply_to("feryirawansyah09@gmail.com".parse().unwrap()) // kalau ini email verified
    .to("ir15y4hh@gmail.com".parse().unwrap())
    .subject(&req.subject)
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
            HttpResponse::Ok().body("Email sent successfully!")
        },
        Err(e) => {
            eprintln!("Failed to send email: {e}");
            HttpResponse::InternalServerError().body(format!("Email failed: {e}"))
        }
    }
}