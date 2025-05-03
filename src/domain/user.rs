use chrono::{DateTime, Utc};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub password: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl User {
    pub fn new(email: String, password: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            email,
            password,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }
}
