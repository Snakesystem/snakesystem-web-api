use actix_web::{post, web, HttpResponse, Responder, Scope};
use lettre::{transport::smtp::authentication::Credentials, Message, SmtpTransport, Transport as _};

use crate::contexts::model::ContactRequest;

pub fn mail_scope() -> Scope {
    
    web::scope("/email")
        .service(contact_form)
}

#[post("/contact")]
async fn contact_form(form: web::Json<ContactRequest>) -> impl Responder {
    let contact = form.into_inner();

    // Email kamu
    let to_email = "feryirawansyah09@gmail.com";
    let from_email = "8cf4d6001@smtp-brevo.com";
    let password = "1qhSR2QDKFBJTLfg"; // Bukan password Gmail biasa!

    // Format email
    let email = Message::builder()
        .from(from_email.parse().unwrap())
        .reply_to(contact.email.parse().unwrap())
        .to(to_email.parse().unwrap())
        .subject(format!("Pesan dari {}", contact.name))
        .header(lettre::message::header::ContentType::TEXT_PLAIN)
        .body(format!(
            "Nama: {}\nEmail: {}\n\nPesan:\n{}",
            contact.name, contact.email, contact.message
        ))
        .unwrap();


    // Konfigurasi SMTP Gmail
    let creds = Credentials::new(from_email.to_string(), password.to_string());

    let mailer = SmtpTransport::starttls_relay("smtp-relay.brevo.com")
        .unwrap()
        .credentials(creds)
        .build();

    match mailer.send(&email) {
        Ok(res) => {
            println!("Email terkirim: {:#?}", res);
            HttpResponse::Ok()
            .json(serde_json::json!({
                "result": true,
                "message": "Logout successful, cookie deleted"
            }))
        },
        Err(e) => {
            println!("Gagal kirim email: {:?}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "result": false,
                "message": "Failed to send email"
            }))
        }
    }
}
