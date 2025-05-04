use redis::RedisResult;

#[async_trait::async_trait]
pub trait RedisRepository {
    async fn set_value(&self, key: &str, value: &str) -> RedisResult<()>;
    async fn get_value(&self, key: &str) -> RedisResult<String>;
    async fn delete_value(&self, key: &str) -> RedisResult<()>;
    async fn exists(&self, key: &str) -> RedisResult<bool>;
    async fn pull_value(&self, key: &str) -> RedisResult<String>;
}
