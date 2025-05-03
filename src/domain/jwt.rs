use std::env;
use std::time::SystemTime;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct AuthClaims {
    sub: String,
    exp: i64,
}

impl AuthClaims {
    pub fn new(sub: String, expiration: SystemTime) -> Self {
        let exp = expiration
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        AuthClaims { sub, exp }
    }
}

pub fn get_jwt_secret() -> Vec<u8> {
    match env::var("JWT_SECRET") {
        Ok(secret) => secret.into_bytes(),
        Err(_) => panic!("JWT_SECRET environment variable is not set"),
    }
}
