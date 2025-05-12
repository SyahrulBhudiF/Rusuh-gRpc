use crate::cfg;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};
use once_cell::sync::Lazy;
use std::sync::Arc;
use tera::{Context, Tera};

static TEMPLATES: Lazy<Arc<Tera>> = Lazy::new(|| {
    let mut tera = Tera::new("util/template/**/*.html").expect("Failed to load templates");
    Arc::new(tera)
});

pub struct EmailSender {
    mailer: AsyncSmtpTransport<Tokio1Executor>,
    sender_email: String,
}

impl EmailSender {
    fn init() -> Self {
        let config: &crate::config::env::EnvConfig = cfg();
        let creds = Credentials::new(config.email_user.clone(), config.email_password.clone());

        let mailer = AsyncSmtpTransport::<Tokio1Executor>::relay(&config.email_host)
            .expect("Failed to create SMTP transport")
            .port(config.email_port.parse().unwrap())
            .credentials(creds)
            .build();

        Self {
            mailer,
            sender_email: config.smtp_from.clone(),
        }
    }

    pub async fn send_otp_email(
        &self,
        recipient: &str,
        otp_code: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut context = Context::new();
        context.insert("otp_code", otp_code);

        let html_body = TEMPLATES
            .render("otp.html", &context)
            .expect("Failed to render OTP template");

        let email = Message::builder()
            .from(self.sender_email.parse()?)
            .to(recipient.parse()?)
            .subject("Your OTP Code")
            .header(lettre::message::header::ContentType::TEXT_HTML)
            .body(html_body)?;

        self.mailer.send(email).await?;
        Ok(())
    }
}

static EMAIL_SENDER: Lazy<EmailSender> = Lazy::new(EmailSender::init);

pub fn email() -> &'static EmailSender {
    &EMAIL_SENDER
}
