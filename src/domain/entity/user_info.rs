use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "user_gender", rename_all = "snake_case")]
pub enum UserGender {
    Male,
    Female,
    PreferNotToSay,
}

impl UserGender {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "male" => Some(Self::Male),
            "female" => Some(Self::Female),
            "prefer-not-to-say" => Some(Self::PreferNotToSay),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            UserGender::Male => "male",
            UserGender::Female => "female",
            UserGender::PreferNotToSay => "prefer-not-to-say",
        }
    }
}
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct UserInfo {
    pub id: Uuid,
    pub user_id: Uuid,
    pub first_name: String,
    pub last_name: String,
    pub gender: UserGender,
    pub birth_date: NaiveDate,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

impl UserInfo {
    pub fn new(
        user_id: Uuid,
        first_name: String,
        last_name: String,
        gender: UserGender,
        birth_date: NaiveDate,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            user_id,
            first_name,
            last_name,
            gender,
            birth_date,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            deleted_at: None,
        }
    }
}
