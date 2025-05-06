use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
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

    pub fn get_jwt_secret(secret: &str) -> String {
        match env::var(secret) {
            Ok(secret) => secret.parse().unwrap(),
            Err(_) => panic!("JWT_SECRET environment variable is not set"),
        }
    }

    async fn create_token(&self, expiration: SystemTime, secret: &str) -> Result<String, Status> {
        let claims = Token::new(self.sub.clone(), expiration);
        encode(
            &Header::new(Algorithm::HS256),
            &claims,
            &EncodingKey::from_secret(Token::get_jwt_secret(secret).as_ref()),
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
            token.create_token(expiration, "ACCESS_SECRET"),
            token.create_token(expiration_refresh, "REFRESH_SECRET"),
        )?;

        Ok((access_token, refresh_token))
    }

    pub fn validate_token(token: &str, secret: &str) -> Result<Token, Status> {
        jsonwebtoken::decode::<Token>(
            token,
            &jsonwebtoken::DecodingKey::from_secret(Token::get_jwt_secret(secret).as_ref()),
            &jsonwebtoken::Validation::new(Algorithm::HS256),
        )
        .map(|data| data.claims)
        .map_err(|e| match e.kind() {
            jsonwebtoken::errors::ErrorKind::ExpiredSignature => {
                Status::unauthenticated("Token expired")
            }
            _ => Status::unauthenticated("Invalid token"),
        })
    }
}
