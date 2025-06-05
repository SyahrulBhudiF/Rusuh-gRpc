use once_cell::sync::Lazy;
use rand::distr::Alphanumeric;
use rand::{Rng, rng};
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::log::{debug, info, warn};

pub struct OtpEmail;

impl OtpEmail {
    pub fn init() -> Self {
        info!("Initializing Email OTP helper.");
        Self {}
    }

    pub fn generate_code(&self, length: usize) -> String {
        let code: String = rng()
            .sample_iter(&Alphanumeric)
            .take(length)
            .map(char::from)
            .collect();
        debug!("Generated new email OTP code: {}", code);
        code
    }

    pub fn create_otp(&self, length: usize, validity_minutes: u64) -> (String, u64) {
        let code = self.generate_code(length);
        let current_time_secs = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("System time is before UNIX EPOCH!")
            .as_secs();
        let expires_at = current_time_secs + (validity_minutes * 60);

        info!(
            "Created OTP code '{}' valid for {} minutes.",
            code, validity_minutes
        );
        (code, expires_at)
    }

    pub fn verify_otp(&self, email: &str, existing: &str, user_input_code: &str) -> bool {
        if existing != user_input_code {
            warn!(
                "OTP verification failed: Code mismatch for recipient {}",
                email
            );
            return false;
        }

        info!("OTP code verified successfully for recipient {}", email);
        true
    }
}

static EMAIL_OTP_HELPER: Lazy<OtpEmail> = Lazy::new(|| OtpEmail::init());

pub fn email_otp() -> &'static OtpEmail {
    &EMAIL_OTP_HELPER
}
