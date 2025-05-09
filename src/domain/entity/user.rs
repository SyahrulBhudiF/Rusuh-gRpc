use serde::{Serialize, Deserialize};
use sqlx::FromRow;
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name="user_status", rename_all="snake_case")]
pub enum UserStatus {
    Active,
    Inactive,
    Suspended,
    Banned
}

impl UserStatus {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "active" => Some(UserStatus::Active),
            "inactive" => Some(UserStatus::Inactive),
            "suspended" => Some(UserStatus::Suspended),
            "banned" => Some(UserStatus::Banned),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            UserStatus::Active => "active",
            UserStatus::Inactive => "inactive",
            UserStatus::Suspended => "suspended",
            UserStatus::Banned => "banned",
        }
    }
}
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub password: String,
    pub status: UserStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl User {
    pub fn new(email: String, password: String, status: UserStatus) -> Self {
        Self {
            id: Uuid::new_v4(),
            email,
            password,
            status,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }
}
