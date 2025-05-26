use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct UserSecurity {
    pub id: Uuid,
    pub user_id: Uuid,
    pub mfa_enabled: bool,
    pub mfa_secret_key: String,
    pub email_verified_at: DateTime<Utc>,
    pub last_password_change: DateTime<Utc>,
    pub password_reset_token: String,
    pub password_reset_expires_at: DateTime<Utc>,
    pub failed_login_attempts: i16,
    pub last_login_failed: DateTime<Utc>,
    pub account_locked_until: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

impl UserSecurity {
    pub fn new(
        user_id: Uuid,
        mfa_enabled: bool,
        mfa_secret_key: String,
        email_verified_at: DateTime<Utc>,
        last_password_change: DateTime<Utc>,
        password_reset_token: String,
        password_reset_expires_at: DateTime<Utc>,
        failed_login_attempts: i16,
        last_login_failed: DateTime<Utc>,
        account_locked_until: DateTime<Utc>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            user_id,
            mfa_enabled,
            mfa_secret_key,
            email_verified_at,
            last_password_change,
            password_reset_token,
            password_reset_expires_at,
            failed_login_attempts,
            last_login_failed,
            account_locked_until,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            deleted_at: None,
        }
    }
}
