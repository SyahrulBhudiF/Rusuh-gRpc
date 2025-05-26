use crate::domain::entity::user_sessions::UserSessions;
use crate::domain::port::db_port::DbPort;
use async_trait::async_trait;

#[async_trait]
pub trait UserSessionPort: DbPort<UserSessions> {}
