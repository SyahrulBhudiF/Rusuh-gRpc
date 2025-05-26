use crate::domain::entity::user::User;
use crate::domain::port::db_port::DbPort;
use async_trait::async_trait;

#[async_trait]
pub trait UserPort: DbPort<User> {}
