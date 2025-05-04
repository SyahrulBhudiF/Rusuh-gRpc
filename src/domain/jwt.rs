use jsonwebtoken::{Algorithm, EncodingKey, Header, encode};
use std::env;
use std::time::{Duration, SystemTime};
use tokio;
use tonic::Status;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Token {
    sub: String,
    exp: i64,
}

impl Token {
    pub fn new(sub: String, expiration: SystemTime) -> Self {
        let exp = expiration
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        Token { sub, exp }
    }

    pub fn get_jwt_secret() -> Vec<u8> {
        match env::var("JWT_SECRET") {
            Ok(secret) => secret.into_bytes(),
            Err(_) => panic!("JWT_SECRET environment variable is not set"),
        }
    }

    async fn create_token(&self, expiration: SystemTime) -> Result<String, Status> {
        let claims = Token::new(self.sub.clone(), expiration);
        encode(
            &Header::new(Algorithm::HS256),
            &claims,
            &EncodingKey::from_secret(Token::get_jwt_secret().as_ref()),
        )
        .map_err(|_| Status::internal("Failed to create token"))
    }

    fn get_duration(env_var: &str, default: u64) -> u64 {
        match env::var(env_var) {
            Ok(val) => val.parse().unwrap_or(default),
            Err(_) => default,
        }
    }

    pub async fn create_tokens(user_id: String) -> Result<(String, String), Status> {
        let access_token_duration = Token::get_duration("ACCESS_TOKEN_DURATION", 3600);
        let refresh_token_duration = Token::get_duration("REFRESH_TOKEN_DURATION", 604800);

        let expiration = SystemTime::now() + Duration::new(access_token_duration, 0);
        let expiration_refresh = SystemTime::now() + Duration::new(refresh_token_duration, 0);

        let token = Token::new(user_id, expiration);

        let (access_token, refresh_token) = tokio::try_join!(
            token.create_token(expiration),
            token.create_token(expiration_refresh)
        )?;

        Ok((access_token, refresh_token))
    }
}
