use std::{collections::HashMap};

use actix_web::{get, post, web, HttpRequest, HttpResponse, Responder, Scope};
use bb8::Pool;
use bb8_tiberius::ConnectionManager;
use handlebars::Handlebars;
use lettre::{transport::smtp::authentication::Credentials, Message, SmtpTransport, Transport as _};
use serde_json::json;
use shuttle_runtime::SecretStore;
use validator::Validate;

use crate::{contexts::model::{ActionResult, ContactRequest, EmailRequest}, services::mail_service::MailService};

pub fn mail_scope() -> Scope {
    
    web::scope("/email")
        .service(contact_form)
        .service(send_campaign)
        .service(preview_email_to)
        .service(preview_email_from)
}

#[post("/contact")]
async fn contact_form(req: HttpRequest, connection: web::Data<Pool<ConnectionManager>>, form: web::Json<EmailRequest>, secrets: web::Data<SecretStore>) -> impl Responder {

    if let Err(err) = form.validate() {
        return HttpResponse::BadRequest().json(json!({
            "result": false,
            "message": "Form tidak valid",
            "error": err
        }));
    }

    let mut sender: EmailRequest = form.clone().into();
    let mut receiver: EmailRequest = form.clone().into();

    let mut result: ActionResult<(), String> = MailService::contact_form(req, connection, form.into_inner()).await;
    
    if result.result {
        let email_sender = MailService::send_email_from({
            sender.recipient = String::from("feryirawansyah09@gmail.com");
            sender
        }, secrets.clone()).await;

        if email_sender.result {
            let email_receiver = MailService::send_email_to({
                receiver.message = String::from("Terima kasih telah menghubungi kami. Kami akan segera menghubungi anda.");
                receiver
            }, secrets).await;
            if email_receiver.result {
                result.result = true;
                result.message = String::from("Email berhasil dikirim.");
            } else {
                result.result = false;
                result.message = String::from("Email gagal dikirim.");
                result.error = Some(email_receiver.message);
            }
        } else {
            result.result = false;
            result.message = String::from("Email gagal dikirim.");
            result.error = Some(email_sender.message);
        }
    } else {
        return HttpResponse::InternalServerError().json(json!({
            "result": result.result,
            "message": result.message,
            "error": result.error
        }));
    }

    HttpResponse::Ok().json(result)
}

#[get("/preview-email-to")]
async fn preview_email_to() -> impl Responder {
    let mut handlebars = Handlebars::new();

    // Baca file mustache-nya
    // let template_str = fs::read_to_string("./templates/mail_to.mustache")
    //     .expect("Gagal baca template");
    let template_str = include_str!("../../templates/mail_to.mustache");

    let res_svg = reqwest::get("https://snakesystem.github.io/svg/email_send.svg").await.unwrap();
    if res_svg.status().is_success() {
        println!("Succes load image");
    }

    let res_fb = reqwest::get("https://cdn-icons-png.flaticon.com/512/733/733547.png").await.unwrap();
    if res_fb.status().is_success() {
        println!("Succes load image");
    }

    let res_linkedin = reqwest::get("https://cdn-icons-png.flaticon.com/512/733/733561.png").await.unwrap();
    if res_linkedin.status().is_success() {
        println!("Succes load image");
    }

    let res_ig = reqwest::get("https://cdn-icons-png.flaticon.com/512/733/733558.png").await.unwrap();
    if res_ig.status().is_success() {
        println!("Succes load image");
    }

    let res_ss = reqwest::get("https://snakesystem.github.io/favicon.ico").await.unwrap();
    if res_ss.status().is_success() {
        println!("Succes load image");
    }

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

#[get("/preview-email-from")]
async fn preview_email_from() -> impl Responder {
    let mut handlebars = Handlebars::new();

    // Baca file mustache-nya
    let template_str = include_str!("../../templates/mail_from.mustache");

    handlebars
        .register_template_string("mail_from", template_str)
        .expect("Gagal daftarin template");

    // Data dummy buat preview
    let mut data = HashMap::new();
    data.insert("subject", "Ini Judul Email Contoh");
    data.insert("message", "Ini isi pesan email yang bisa kamu ubah dan lihat hasilnya langsung di browser.");
    data.insert("recipient", "ir15y4hh@gmail.com");
    data.insert("name", "Dede Sukron");
    data.insert("url", "http://localhost:8000/api/v1/auth/activation/hdhshsdbshdbshd");

    let rendered = handlebars.render("mail_from", &data).unwrap();

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