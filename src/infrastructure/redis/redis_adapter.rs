use crate::config::redis::RedisClient;
use crate::domain::port::redis_port::RedisPort;
use redis::{AsyncCommands, RedisResult};
use tonic::Status;
use tracing::log::error;

pub struct RedisAdapter {
    pub redis: RedisClient,
}

impl RedisAdapter {
    pub fn new() -> Self {
        let redis = RedisClient::new().expect("Failed to create Redis client");
        RedisAdapter { redis }
    }
}

#[async_trait::async_trait]
impl RedisPort for RedisAdapter {
    async fn set_value(&self, key: &str, value: &str) -> RedisResult<()> {
        let mut conn = self.redis.client.get_multiplexed_tokio_connection().await?;
        conn.set(key, value).await
    }

    async fn get_value(&self, key: &str) -> RedisResult<Option<String>> {
        let mut conn = self.redis.client.get_multiplexed_tokio_connection().await?;
        conn.get(key).await
    }

    async fn delete_value(&self, key: &str) -> RedisResult<()> {
        let mut conn = self.redis.client.get_multiplexed_tokio_connection().await?;
        conn.del(key).await
    }

    async fn exists(&self, key: &str) -> RedisResult<Option<bool>> {
        let mut conn = self.redis.client.get_multiplexed_tokio_connection().await?;
        let result: i32 = conn.exists(key).await?;
        Ok(Some(result > 0))
    }

    async fn pull_value(&self, key: &str) -> RedisResult<Option<String>> {
        let mut conn = self.redis.client.get_multiplexed_tokio_connection().await?;
        conn.get_del(key).await
    }

    async fn blacklist_token(&self, token: &str) -> RedisResult<()> {
        let mut conn = self.redis.client.get_multiplexed_tokio_connection().await?;
        conn.set(token, "BLACKLISTED").await
    }

    async fn ensure_not_blacklisted(&self, token: &str) -> Result<(), Status> {
        let mut conn = self
            .redis
            .client
            .get_multiplexed_tokio_connection()
            .await
            .map_err(|e| {
                error!("Failed to get Redis connection: {}", e);
                Status::internal("Failed to access Redis")
            })?;

        match conn.get::<_, Option<String>>(token).await {
            Ok(Some(ref value)) if value == "BLACKLISTED" => Err(Status::unauthenticated(
                "Token already invalidated or blacklisted",
            )),
            Ok(_) => Ok(()),
            Err(e) => {
                error!("Redis error while checking blacklist: {}", e);
                Err(Status::internal("Internal error"))
            }
        }
    }
}
