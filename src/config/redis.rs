use crate::cfg;
use redis::{Client, RedisResult}; // Import cfg from the crate root

pub struct RedisClient {
    pub client: Client,
}

impl RedisClient {
    pub fn new() -> RedisResult<Self> {
        let config = cfg();
        let host = &config.redis_host;
        let port = config.redis_port;
        let password = config.redis_password.as_deref(); // Use as_deref for Option<String>

        let url = if let Some(pw) = password {
            format!("redis://:{}@{}:{}/", pw, host, port)
        } else {
            format!("redis://{}:{}/", host, port)
        };

        let client = Client::open(url)?;

        Ok(Self { client })
    }
}
