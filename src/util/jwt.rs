use crate::cfg; // Import cfg from the crate root
use jsonwebtoken::{Algorithm, EncodingKey, Header, encode};
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

    async fn create_token(
        &self,
        expiration: SystemTime,
        secret_key: &str,
    ) -> Result<String, Status> {
        let claims = Token::new(self.sub.clone(), expiration);
        encode(
            &Header::new(Algorithm::HS256),
            &claims,
            &EncodingKey::from_secret(secret_key.as_ref()),
        )
        .map_err(|_| Status::internal("Failed to create token"))
    }

    pub async fn create_tokens(user_id: String) -> Result<(String, String), Status> {
        let config = cfg();
        let access_token_duration_secs = config.access_token_duration as u64;
        let refresh_token_duration_secs = config.refresh_token_duration as u64;

        let expiration = SystemTime::now() + Duration::new(access_token_duration_secs, 0);
        let expiration_refresh = SystemTime::now() + Duration::new(refresh_token_duration_secs, 0);

        let access_token_claims = Token::new(user_id.clone(), expiration);
        let refresh_token_claims = Token::new(user_id, expiration_refresh);

        let (access_token, refresh_token) = tokio::try_join!(
            access_token_claims.create_token(expiration, &config.access_secret),
            refresh_token_claims.create_token(expiration_refresh, &config.refresh_secret),
        )?;

        Ok((access_token, refresh_token))
    }

    pub fn validate_token(token_str: &str, secret_key: &str) -> Result<Token, Status> {
        jsonwebtoken::decode::<Token>(
            token_str,
            &jsonwebtoken::DecodingKey::from_secret(secret_key.as_ref()),
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
