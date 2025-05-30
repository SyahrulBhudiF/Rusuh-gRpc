use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct UserSecurity {
    pub id: Uuid,
    pub user_id: Uuid,
    pub mfa_secret_key: Option<String>,
    pub email_verified_at: Option<DateTime<Utc>>,
    pub last_password_change: Option<DateTime<Utc>>,
    pub account_locked_until: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

impl UserSecurity {
    pub fn new(
        user_id: Uuid,
        mfa_secret_key: Option<String>,
        email_verified_at: Option<DateTime<Utc>>,
        last_password_change: Option<DateTime<Utc>>,
        account_locked_until: Option<DateTime<Utc>>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            user_id,
            mfa_secret_key,
            email_verified_at,
            last_password_change,
            account_locked_until,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            deleted_at: None,
        }
    }
}
