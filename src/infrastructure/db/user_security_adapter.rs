use crate::domain::entity::user_security::UserSecurity;
use crate::domain::port::db_port::DbPort;
use async_trait::async_trait;
use sqlx::Error;
use uuid::Uuid;

pub struct UserSecurityAdapter {
    pub pool: sqlx::PgPool,
}

#[async_trait]
impl DbPort<UserSecurity> for UserSecurityAdapter {
    async fn save(&self, data: &UserSecurity) -> Result<(), Error> {
        sqlx::query(
            "INSERT INTO user_security (id, user_id, mfa_enabled, mfa_secret_key, email_verified_at, last_password_change, password_reset_token, password_reset_expires_at, failed_login_attempts, last_login_failed, account_locked_until, created_at, updated_at)\
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
        )
        .bind(data.id)
        .bind(data.user_id)
        .bind(data.mfa_enabled)
        .bind(data.mfa_secret_key)
        .bind(data.email_verified_at)
        .bind(data.last_password_change)
        .bind(data.password_reset_token)
        .bind(data.password_reset_expires_at)
        .bind(data.failed_login_attempts)
        .bind(data.last_login_attempts)
        .bind(data.account_locked_until)
        .bind(data.created_at)
        .bind(data.updated_at)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn find_by_id(&self, id: Uuid) -> Result<UserSecurity, Error> {
        let result = sqlx::query_as::<_, UserSecurity>(
            "SELECT d, user_id, mfa_enabled, mfa_secret_key, email_verified_at, last_password_change, created_at, updated_at FROM user_sessions WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(result)
    }

    async fn find_by_coll(&self, coll:&str, value: &str) -> Result<Option<UserSecurity>, Error> {
        let query = match coll {
            "mfa_enabled" => {
                "SELECT id, user_id, mfa_enabled, mfa_secret_key, email_verified_at, last_password_change, created_at, updated_at FROM user_security WHERE mfa_enabled = $1"
            }
            "user_id" => {
                "SELECT id, user_id, mfa_enabled, mfa_secret_key, email_verified_at, last_password_change, created_at, updated_at FROM user_security WHERE user_id = $1"
            }
            "id" => "SELECT id, user_id, mfa_enabled, mfa_secret_key, email_verified_at, last_password_change, created_at, updated_at FROM user_security WHERE id = $1",
            _ => return Err(Error::RowNotFound),
        };
        let result = sqlx::query_as::<_, UserSecurity>(query)
        .bind(value)
        .fetch_optional(&self.pool)
        .await?;

        Ok(result)
    }

    async fn update(&self, id: Uuid, data: &UserSecurity) -> Result<(), Error> {
        sqlx::query(
            "UPDATE user_security\
            SET mfa_enabled = $1, email_verified_at = $2, failed_login_attempts = $3 last_password_change = $4, account_locked_until = $5, updated_at = $6 \
            WHERE id = $7",
        )
        .bind(&data.mfa_enabled)
        .bind(&data.email_verified_at)
        .bind(&data.failed_login_attempts)
        .bind(&data.last_password_change)
        .bind(data.account_locked_until)
        .bind(&data.updated_at)
        .bind(id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }
    
    async fn delete(&self, id: Uuid) -> Result<(), Error> {
        let query = sqlx::query("DELETE FROM user_security WHERE id = $1").bind(id);
        
        query.execute(&self.pool).await?;
        Ok(())
    }
}