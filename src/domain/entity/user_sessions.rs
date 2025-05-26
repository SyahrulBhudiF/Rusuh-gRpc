use serde::{Serialize, Deserialize};
use sqlx::FromRow;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use std::net::IpAddr;

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct UserSessions {
    pub id: Uuid,
    pub user_id: Uuid,
    pub login_ip: IpAddr,
    pub login_device: String,
    pub login_location: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl UserSessions {
    pub fn new(user_id: Uuid, login_ip: IpAddr, login_device: String, login_location: String) ->Self{
        Self{
            id: Uuid::new_v4(),
            user_id,
            login_ip,
            login_device,
            login_location,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }
}