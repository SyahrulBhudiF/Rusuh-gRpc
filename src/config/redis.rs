use redis::{Client, RedisResult};
use std::env;

pub struct RedisClient {
    pub client: Client,
}

impl RedisClient {
    pub fn new() -> RedisResult<Self> {
        let host = env::var("REDIS_HOST").unwrap_or_else(|_| "localhost".to_string());
        let port = env::var("REDIS_PORT").unwrap_or_else(|_| "6379".to_string());
        let password = env::var("REDIS_PASSWORD").ok();

        let url = if let Some(pw) = password {
            format!("redis://:{}@{}:{}/", pw, host, port)
        } else {
            format!("redis://{}:{}/", host, port)
        };

        let client = Client::open(url)?;

        Ok(Self { client })
    }
}
