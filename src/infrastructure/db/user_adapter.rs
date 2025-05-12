use crate::domain::entity::user::User;
use crate::domain::port::db_port::DbPort;
use async_trait::async_trait;
use sqlx::Error;
use uuid::Uuid;

pub struct UserAdapter {
    pub pool: sqlx::PgPool,
}

#[async_trait]
impl DbPort<User> for UserAdapter {
    async fn save(&self, data: &User) -> Result<(), Error> {
        sqlx::query(
            "INSERT INTO users (id, email, password, created_at, updated_at) 
             VALUES ($1, $2, $3, $4, $5)",
        )
        .bind(data.id)
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
            "SELECT id, email, password, created_at, updated_at FROM users WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(result)
    }

    async fn find_by_coll(&self, coll: &str, value: &str) -> Result<Option<User>, Error> {
        let query = match coll {
            "email" => {
                "SELECT id, email, password, created_at, updated_at FROM users WHERE email = $1"
            }
            "id" => "SELECT id, email, password, created_at, updated_at FROM users WHERE id = $1",
            _ => return Err(Error::RowNotFound),
        };

        let result = sqlx::query_as::<_, User>(query)
            .bind(value)
            .fetch_optional(&self.pool)
            .await?;

        Ok(result)
    }

    async fn update(&self, id: Uuid, data: &User) -> Result<(), Error> {
        sqlx::query(
            "UPDATE users
             SET email = $1, password = $2, updated_at = $3
             WHERE id = $4",
        )
        .bind(&data.email)
        .bind(&data.password)
        .bind(data.updated_at)
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
