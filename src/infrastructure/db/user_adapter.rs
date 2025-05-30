use crate::domain::entity::user::{User, UserStatus};
use crate::domain::port::db::user_port::UserPort;
use crate::domain::port::db_port::DbPort;
use async_trait::async_trait;
use chrono::Utc;
use sqlx::Error;
use tracing::info;
use uuid::Uuid;

pub struct UserAdapter {
    pub pool: sqlx::PgPool,
}

impl UserAdapter {
    pub fn new(pool: sqlx::PgPool) -> Self {
        UserAdapter { pool }
    }
}

#[async_trait]
impl DbPort<User> for UserAdapter {
    async fn save(&self, data: &User) -> Result<(), Error> {
        sqlx::query(
            "INSERT INTO users (id, name, email, password, created_at, updated_at)
             VALUES ($1, $2, $3, $4, $5, $6)",
        )
        .bind(data.id)
        .bind(&data.name)
        .bind(&data.email)
        .bind(&data.password)
        .bind(data.created_at)
        .bind(data.updated_at)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<User>, Error> {
        let result = sqlx::query_as::<_, User>(
            "SELECT id, name, email, password, status,created_at, updated_at, deleted_at FROM users WHERE id = $1 AND deleted_at IS NULL",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(result)
    }

    async fn find_by_coll(&self, coll: &str, value: &str) -> Result<Option<User>, Error> {
        let query = match coll {
            "email" => {
                "SELECT id, name, email, password, status, created_at, updated_at, deleted_at FROM users WHERE email = $1 AND deleted_at IS NULL"
            }
            "id" => {
                "SELECT id, name, email, password, status, created_at, updated_at, deleted_at FROM users WHERE id = $1 AND deleted_at IS NULL"
            }
            _ => return Err(Error::RowNotFound),
        };

        let result = sqlx::query_as::<_, User>(query)
            .bind(value)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| {
                info!("Error finding user by collection: {}", e);
                e
            })?;

        Ok(result)
    }

    async fn update(&self, id: Uuid, data: &User) -> Result<(), Error> {
        sqlx::query(
            "UPDATE users
             SET email = $1, password = $2, updated_at = $3, name = $4
             WHERE id = $4 AND deleted_at IS NULL",
        )
        .bind(&data.email)
        .bind(&data.password)
        .bind(data.updated_at)
        .bind(&data.name)
        .bind(id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn delete(&self, id: Uuid) -> Result<(), Error> {
        let query = sqlx::query("DELETE FROM users WHERE id = $1").bind(id);

        query.execute(&self.pool).await?;
        Ok(())
    }
}

#[async_trait]
impl UserPort for UserAdapter {
    async fn verify_email(&self, id: Uuid) -> Result<(), Error> {
        let mut transaction = self.pool.begin().await?;

        sqlx::query(
            "UPDATE users SET status = $1, updated_at = $2 WHERE id = $3 AND deleted_at IS NULL",
        )
        .bind(UserStatus::Active)
        .bind(Utc::now())
        .bind(id)
        .execute(&mut *transaction)
        .await?;

        sqlx::query(
            "INSERT INTO user_security (
        id,
        user_id,
        email_verified_at,
        updated_at,
        created_at,
        deleted_at
    ) VALUES ($1, $2, $3, $4, $5, NULL)",
        )
        .bind(Uuid::new_v4())
        .bind(id)
        .bind(Some(Utc::now()))
        .bind(Utc::now())
        .bind(Utc::now())
        .execute(&mut *transaction)
        .await?;

        transaction.commit().await?;

        info!("Email verified for user: {}", id.to_string());

        Ok(())
    }
}
