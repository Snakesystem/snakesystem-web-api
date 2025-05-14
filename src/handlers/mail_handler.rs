use std::{collections::HashMap, fs};

use actix_web::{post, get, web, HttpResponse, Responder, Scope};
use handlebars::Handlebars;
use lettre::{transport::smtp::authentication::Credentials, Message, SmtpTransport, Transport as _};

use crate::{contexts::model::{ContactRequest, EmailRequest}, services::mail_service::MailService};

pub fn mail_scope() -> Scope {
    
    web::scope("/email")
        .service(contact_form)
        .service(send_campaign)
        .service(preview_email)
}

#[post("/contact")]
async fn contact_form(form: web::Json<EmailRequest>) -> impl Responder {
    let mut sender: EmailRequest = form.clone().into();
    let mut receiver = form.into_inner();
    
    let mut result = MailService::send_email({
        sender.recipient = String::from("feryirawansyah09@gmail.com");
        sender
    });

    if result.result {
        result = MailService::send_email({
            receiver.message = String::from("Terima kasih telah menghubungi kami. Kami akan segera menghubungi anda.");
            receiver
        });
        if result.result {
            HttpResponse::Ok().json(result)
        } else {
            HttpResponse::InternalServerError().json(result)
        }
            
    } else {
        HttpResponse::InternalServerError().json(result)
    }
}

#[get("/preview-email")]
async fn preview_email() -> impl Responder {
    let mut handlebars = Handlebars::new();

    // Baca file mustache-nya
    let template_str = fs::read_to_string("templates/mail_to.mustache")
        .expect("Gagal baca template");

    handlebars
        .register_template_string("mail_to", template_str)
        .expect("Gagal daftarin template");

    // Data dummy buat preview
    let mut data = HashMap::new();
    data.insert("subject", "Ini Judul Email Contoh");
    data.insert("message", "Ini isi pesan email yang bisa kamu ubah dan lihat hasilnya langsung di browser.");
    data.insert("recipient", "ir15y4hh@gmail.com");
    data.insert("name", "Dede Sukron");
    data.insert("url", "http://localhost:8000/api/v1/auth/activation/hdhshsdbshdbshd");

    let rendered = handlebars.render("mail_to", &data).unwrap();

    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(rendered)
}

#[post("/send-campaign")]
async fn send_campaign(req: web::Json<ContactRequest>) -> impl Responder {
    let smtp_username = ""; // Login dari Brevo
    let smtp_password = "";           // Ambil dari SMTP Brevo
    let smtp_server = "";

    let email = Message::builder()
    .from("".parse().unwrap())
    .reply_to("".parse().unwrap()) // kalau ini email verified
    .to("".parse().unwrap())
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