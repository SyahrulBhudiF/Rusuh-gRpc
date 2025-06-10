use crate::domain::entity::user::User;
use crate::domain::port::db_port::DbPort;
use async_trait::async_trait;
use uuid::Uuid;

#[async_trait]
pub trait UserPort: DbPort<User> {
    async fn verify_email(&self, id: Uuid) -> Result<(), sqlx::Error>;
    async fn update_password(&self, id: Uuid, data: &User) -> Result<(), sqlx::Error>;
}
