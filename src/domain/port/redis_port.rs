use redis::RedisResult;
use tonic::Status;

#[async_trait::async_trait]
pub trait RedisPort {
    async fn set_value(&self, key: &str, value: &str) -> RedisResult<()>;
    async fn get_value(&self, key: &str) -> RedisResult<Option<String>>;
    async fn delete_value(&self, key: &str) -> RedisResult<()>;
    async fn exists(&self, key: &str) -> RedisResult<Option<bool>>;
    async fn pull_value(&self, key: &str) -> RedisResult<Option<String>>;
    async fn blacklist_token(&self, token: &str) -> RedisResult<()>;
    async fn ensure_not_blacklisted(&self, token: &str) -> Result<(), Status>;
}
