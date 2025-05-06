use crate::config::redis::RedisClient;
use crate::domain::redis_repository::RedisRepository;
use redis::{AsyncCommands, RedisResult};

pub struct RedisRepositoryImpl {
    pub redis: RedisClient,
}

impl RedisRepositoryImpl {
    pub fn new() -> Self {
        let redis = RedisClient::new().expect("Failed to create Redis client");
        RedisRepositoryImpl { redis }
    }
}

#[async_trait::async_trait]
impl RedisRepository for RedisRepositoryImpl {
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
}
