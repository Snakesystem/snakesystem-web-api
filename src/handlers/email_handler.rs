use actix_web::{post, web, HttpResponse, Responder, Scope};
use lettre::{transport::smtp::authentication::Credentials, Message, SmtpTransport, Transport as _};

use crate::contexts::model::ContactRequest;

pub fn email_scope() -> Scope {
    
    web::scope("/email")
        .service(contact_form)
}

#[post("/contact")]
async fn contact_form(form: web::Json<ContactRequest>) -> impl Responder {
    let contact = form.into_inner();

    // Email kamu
    let to_email = "feryirawansyah09@gmail.com";
    let from_email = "ir15y4hh@gmail.com";
    let password = ""; // Bukan password Gmail biasa!

    // Format email
    let email = Message::builder()
        .from(from_email.parse().unwrap())
        .reply_to(contact.email.parse().unwrap())
        .to(to_email.parse().unwrap())
        .subject(format!("Pesan dari {}", contact.name))
        .body(contact.message)
        .unwrap();

    // Konfigurasi SMTP Gmail
    let creds = Credentials::new(from_email.to_string(), password.to_string());

    let mailer = SmtpTransport::relay("smtp.gmail.com")
        .unwrap()
        .credentials(creds)
        .build();

    match mailer.send(&email) {
        Ok(_) => HttpResponse::Ok()
        .json(serde_json::json!({
            "result": true,
            "message": "Logout successful, cookie deleted"
        })),
        Err(e) => {
            println!("Gagal kirim email: {:?}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "result": false,
                "message": "Failed to send email"
            }))
        }
    }
}
