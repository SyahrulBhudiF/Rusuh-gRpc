use crate::domain::entity::user_security::UserSecurity;
use crate::domain::port::db_port::DbPort;
use async_trait::async_trait;
use sqlx::{Error, Pool, Postgres};
use uuid::Uuid;

pub struct UserSecurityAdapter {
    pub pool: sqlx::PgPool,
}

impl UserSecurityAdapter {
    pub(crate) fn new(p: Pool<Postgres>) -> Self {
        UserSecurityAdapter { pool: p }
    }
}

#[async_trait]
impl DbPort<UserSecurity> for UserSecurityAdapter {
    async fn save(&self, data: &UserSecurity) -> Result<(), Error> {
        sqlx::query(
            "INSERT INTO user_security (id, user_id, mfa_secret_key, email_verified_at, last_password_change, account_locked_until, created_at, updated_at)\
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
        )
        .bind(data.id)
        .bind(data.user_id)
        .bind(&data.mfa_secret_key)
        .bind(data.email_verified_at)
        .bind(data.last_password_change)
        .bind(data.account_locked_until)
        .bind(data.created_at)
        .bind(data.updated_at)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<UserSecurity>, Error> {
        let result = sqlx::query_as::<_, UserSecurity>(
            "SELECT id, user_id, mfa_secret_key, email_verified_at, last_password_change, created_at, updated_at FROM user_security WHERE id = $1 AND deleted_at IS NULL",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(result)
    }

    async fn find_by_coll(&self, coll: &str, value: &str) -> Result<Option<UserSecurity>, Error> {
        let query = match coll {
            "user_id" => {
                "SELECT id, user_id, mfa_secret_key, email_verified_at, last_password_change, created_at, updated_at FROM user_security WHERE user_id = $1 AND deleted_at IS NULL"
            }
            "id" => {
                "SELECT id, user_id, mfa_secret_key, email_verified_at, last_password_change, created_at, updated_at FROM user_security WHERE id = $1 AND deleted_at IS NULL"
            }
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
            "UPDATE user_security
            SET email_verified_at = $1, last_password_change = $2, account_locked_until = $3, updated_at = $4, mfa_secret_key = $5 \
            WHERE id = $6 AND deleted_at IS NULL",
        )
        .bind(&data.email_verified_at)
        .bind(&data.last_password_change)
        .bind(data.account_locked_until)
        .bind(&data.updated_at)
        .bind(&data.mfa_secret_key)
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
