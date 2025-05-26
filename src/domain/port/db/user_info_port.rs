use crate::domain::entity::user_info::UserInfo;
use crate::domain::port::db_port::DbPort;
use async_trait::async_trait;

#[async_trait]
pub trait UserInfoPort: DbPort<UserInfo> {}
