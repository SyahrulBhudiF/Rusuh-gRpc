use crate::cfg;
use base32::{Alphabet, encode};
use once_cell::sync::Lazy;
use otpauth::TOTP;
use std::time::{SystemTime, UNIX_EPOCH};

pub struct Totp {
    issuer: String,
}

impl Totp {
    pub fn init() -> Self {
        Self {
            issuer: cfg().app_name.clone(),
        }
    }

    pub fn generate_secret(&self) -> String {
        let random_bytes: [u8; 32] = rand::random();
        encode(Alphabet::Rfc4648 { padding: false }, &random_bytes)
    }

    pub fn generate_uri(&self, account: &str, secret: &str) -> Option<String> {
        let totp = TOTP::from_base32(secret);
        Some(
            totp.expect("Error generate OTP")
                .to_uri(account, self.issuer.as_str()),
        )
    }

    pub fn generate_code(&self, secret: &str) -> Option<String> {
        let totp = TOTP::from_base32(secret);
        let now = SystemTime::now().duration_since(UNIX_EPOCH).ok()?.as_secs();
        Some(totp?.generate(300, now).to_string())
    }

    pub fn verify_code(&self, secret: &str, code: u32) -> bool {
        if let Some(totp) = TOTP::from_base32(secret.to_string()) {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();
            totp.verify(code, 30, now)
        } else {
            false
        }
    }
}

static OTP_HELPER: Lazy<Totp> = Lazy::new(|| Totp::init());

pub fn totp() -> &'static Totp {
    &OTP_HELPER
}
