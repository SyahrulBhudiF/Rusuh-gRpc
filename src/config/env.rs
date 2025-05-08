use dotenv::dotenv;
use once_cell::sync::Lazy;
use std::env;

#[derive(Debug)]
pub struct EnvConfig {
    pub database_url: String,
    pub access_secret: String,
    pub refresh_secret: String,
    pub access_token_duration: i64,
    pub refresh_token_duration: i64,
    pub redis_host: String,
    pub redis_port: u16,
    pub redis_password: Option<String>,
    pub email_host: String,
    pub email_user: String,
    pub email_password: String,
    pub email_port: String,
    pub server_address: String,
    pub smtp_from: String,
    pub app_name: String,
    pub secret_key: String,
}

impl EnvConfig {
    fn init() -> Self {
        dotenv().ok();

        EnvConfig {
            database_url: env::var("DATABASE_URL").expect("DATABASE_URL must be set"),

            access_secret: env::var("ACCESS_SECRET").expect("ACCESS_SECRET must be set"),
            refresh_secret: env::var("REFRESH_SECRET").expect("REFRESH_SECRET must be set"),
            access_token_duration: env::var("ACCESS_TOKEN_DURATION")
                .expect("ACCESS_TOKEN_DURATION must be set")
                .parse()
                .expect("ACCESS_TOKEN_DURATION must be a valid integer"),
            refresh_token_duration: env::var("REFRESH_TOKEN_DURATION")
                .expect("REFRESH_TOKEN_DURATION must be set")
                .parse()
                .expect("REFRESH_TOKEN_DURATION must be a valid integer"),

            redis_host: env::var("REDIS_HOST").expect("REDIS_HOST must be set"),
            redis_port: env::var("REDIS_PORT")
                .expect("REDIS_PORT must be set")
                .parse()
                .expect("REDIS_PORT must be a valid u16"),
            redis_password: env::var("REDIS_PASSWORD").ok(),

            email_host: env::var("EMAIL_HOST").expect("EMAIL_HOST must be set"),
            email_user: env::var("EMAIL_USER").expect("EMAIL_USER must be set"),
            email_password: env::var("EMAIL_PASSWORD").expect("EMAIL_PASSWORD must be set"),
            email_port: env::var("EMAIL_PORT").unwrap_or_else(|_| "587".to_string()),

            server_address: env::var("SERVER_ADDRESS")
                .unwrap_or_else(|_| "0.0.0.0:50051".to_string()),
            app_name: env::var("APP_NAME").unwrap_or_else(|_| "MyApp".to_string()),
            secret_key: env::var("SECRET_KEY").expect("SECRET_KEY must be set"),
            smtp_from: env::var("SMTP_FROM").unwrap_or_else(|_| "noreply@example.com".to_string()),
        }
    }
}

pub static ENV_CONFIG: Lazy<EnvConfig> = Lazy::new(EnvConfig::init);

pub fn cfg() -> &'static EnvConfig {
    &ENV_CONFIG
}
