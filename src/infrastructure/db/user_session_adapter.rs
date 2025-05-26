use crate::domain::entity::user_sessions::UserSessions;
use crate::domain::port::db_port::DbPort;
use async_trait::async_trait;
use sqlx::{Error, Encode, Decode};
use uuid::Uuid;

pub struct UserSessionAdapter {
    pub pool: sqlx::PgPool,
}

#[async_trait]
impl DbPort<UserSessions> for UserSessionAdapter {
    async fn save(&self, data: &UserSessions) -> Result<(), Error> {
        sqlx::query(
            "INSERT INTO user_sessions (id, user_id, login_ip, login_device, login_location, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7)",
        )
            .bind(data.id)
            .bind(data.user_id)
            .bind(data.login_ip)
            .bind(&data.login_device)
            .bind(&data.login_location)
            .bind(data.created_at)
            .bind(data.updated_at)
            .execute(&self.pool)
            .await?;
            Ok(())
    }
    
    async fn find_by_id(&self, id: Uuid) -> Result<Option<UserSessions>, Error> {
        let result = sqlx::query_as::<_, UserSessions>(
            "SELECT id, user_id, login_ip, login_device, login_location, created_at, updated_at FROM user_sessions WHERE id = $1",
        )
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;
        
        Ok(result)
    }

    async fn find_by_coll(&self, coll: &str, value: &str) -> Result<Option<UserSessions>, Error> {
        let query = match coll {
            "login_device" => {
                "SELECT id, user_id, login_ip, login_device, login_location FROM user_sessions WHERE login_device = $1"
            }
            
            "login_location" => {
                "SELECT id, user_id, login_ip, login_device, login_location FROM user_sessions WHERE login_location = $1"
            }
            
            "id" => "SELECT id, user_id, login_ip, login_device, login_location FROM user_sessions WHERE id = $1",
            _ => return Err(Error::RowNotFound),
        };
        
        let result = sqlx::query_as::<_, UserSessions>(query)
        .bind(value)
            .fetch_optional(&self.pool)
            .await?;
        
        Ok(result)
    }
    
    async fn update(&self, id: Uuid, data: &UserSessions) -> Result<(), Error> {
        sqlx::query(
            "UPDATE user_sessions
                SET login_ip = login_ip + $1, login_device = login_device + $2, login_location = login_location + $3, updated_at = $4
                WHERE id = $5",
        )
            .bind(data.login_ip)
            .bind(&data.login_device)
            .bind(&data.login_location)
            .bind(data.updated_at)
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
    
    async fn delete(&self, id: Uuid) -> Result<(), Error> {
        let query = sqlx::query("DELETE FROM user_session WHERE id = $1").bind(id);
        
        query.execute(&self.pool).await?;
        Ok(())
    }
}
