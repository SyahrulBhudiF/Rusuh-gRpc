use async_trait::async_trait;
use sqlx::Error;
use uuid::Uuid;

#[async_trait]
pub trait DbPort<T>: Send + Sync {
    async fn save(&self, data: &T) -> Result<(), Error>;
    async fn find_by_id(&self, id: Uuid) -> Result<Option<T>, Error>;
    async fn find_by_coll(&self, coll: &str, value: &str) -> Result<Option<T>, Error>;
    async fn update(&self, id: Uuid, data: &T) -> Result<(), Error>;
    async fn delete(&self, id: Uuid) -> Result<(), Error>;
}
