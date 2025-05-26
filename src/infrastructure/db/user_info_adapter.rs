use crate::domain::entity::user_info::UserInfo;
use crate::domain::port::db_port::DbPort;
use async_trait::async_trait;
use sqlx::Error;
use uuid::Uuid;

pub struct UserInfoAdapter {
    pub pool: sqlx::PgPool,
}

#[async_trait]
impl DbPort<UserInfo> for UserInfoAdapter {
    async fn save(&self, data: &UserInfo) -> Result<(), Error> {
        sqlx::query(
            "INSERT INTO user_info (id, user_id, first_name, last_name, gender, birth_date, created_at, updated_at) 
            VALUES ($1, $2, $3, $4, $5, $6, $7)",
        )
        .bind(&data.id)
        .bind(&data.user_id)
        .bind(&data.first_name)
        .bind(&data.last_name)
        .bind(&data.gender)
        .bind(&data.birth_date)
        .bind(&data.created_at)
        .bind(&data.updated_at)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<UserInfo>, Error> {
        let result = sqlx::query_as::<_, UserInfo>("SELECT * FROM user_info WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;

        Ok(result)
    }

    async fn find_by_coll(&self, coll: &str, value: &str) -> Result<Option<UserInfo>, Error> {
        let query = match coll {
            "user_id" => "SELECT * FROM user_info WHERE user_id = $1",
            "first_name" => "SELECT * FROM user_info WHERE first_name = $1",
            "last_name" => "SELECT * FROM user_info WHERE last_name = $1",
            "gender" => "SELECT * FROM user_info WHERE gender = $1",
            "id" => "SELECT * FROM user_info WHERE id = $1",
            &_ => return Err(Error::RowNotFound),
        };

        let result = sqlx::query_as::<_, UserInfo>(query)
            .bind(value)
            .fetch_optional(&self.pool)
            .await?;

        Ok(result)
    }

    async fn update(&self, id: Uuid, data: &UserInfo) -> Result<(), Error> {
        sqlx::query(
            "UPDATE user_info
            SET first_name = $1, last_name = $2, updated_at = $3 WHERE id = $4",
        )
        .bind(&data.first_name)
        .bind(&data.last_name)
        .bind(data.updated_at)
        .bind(id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn delete(&self, id: Uuid) -> Result<(), Error> {
        let query = sqlx::query("DELETE FROM user_info WHERE id = $1").bind(id);

        query.execute(&self.pool).await?;
        Ok(())
    }
}
