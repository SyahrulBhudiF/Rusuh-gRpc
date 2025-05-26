use crate::domain::entity::user_security::UserSecurity;
use crate::domain::port::db_port::DbPort;
use async_trait::async_trait;

#[async_trait]
pub trait UserSecurityPort: DbPort<UserSecurity> {}
