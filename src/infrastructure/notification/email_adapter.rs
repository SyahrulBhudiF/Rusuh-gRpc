use crate::cfg;
use lettre::transport::smtp::authentication::Credentials;
use lettre::transport::smtp::client::{Tls, TlsParameters};
use lettre::{AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};
use once_cell::sync::Lazy;
use std::sync::Arc;
use tera::{Context, Tera};
use tracing::log::{error, info};

static TEMPLATES: Lazy<Arc<Tera>> = Lazy::new(|| match Tera::new("src/util/template/*.html") {
    Ok(tera) => {
        info!("Templates loaded successfully.");
        Arc::new(tera)
    }
    Err(e) => {
        error!("Failed to load templates: {}", e);
        panic!("Template loading failed");
    }
});

pub struct EmailSender {
    mailer: AsyncSmtpTransport<Tokio1Executor>,
    sender_email: String,
}

impl EmailSender {
    fn init() -> Self {
        let config: &crate::config::env::EnvConfig = cfg();
        let creds = Credentials::new(config.email_user.clone(), config.email_password.clone());

        let tls_parameters = TlsParameters::builder(config.email_host.clone())
            .build()
            .expect("Failed to build TLS parameters");

        let mailer = AsyncSmtpTransport::<Tokio1Executor>::relay(&config.email_host)
            .expect("Failed to create SMTP transport")
            .port(config.email_port.parse().unwrap())
            .credentials(creds)
            .tls(Tls::Required(tls_parameters))
            .build();

        info!("Email sender initialized with host: {}", config.email_host);

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

        let html_body = TEMPLATES.render("otp.html", &context).map_err(|e| {
            error!("Failed to render OTP template: {}", e);
            Box::new(e) as Box<dyn std::error::Error>
        })?;

        let email = Message::builder()
            .from(self.sender_email.parse()?)
            .to(recipient.parse()?)
            .subject("Your OTP Code")
            .header(lettre::message::header::ContentType::TEXT_HTML)
            .body(html_body)
            .map_err(|e| {
                error!("Failed to build email: {}", e);
                Box::new(e) as Box<dyn std::error::Error>
            })?;

        info!("Sending OTP email to: {}", recipient);

        self.mailer.send(email).await?;
        Ok(())
    }
}

static EMAIL_SENDER: Lazy<EmailSender> = Lazy::new(EmailSender::init);

pub fn email() -> &'static EmailSender {
    &EMAIL_SENDER
}
