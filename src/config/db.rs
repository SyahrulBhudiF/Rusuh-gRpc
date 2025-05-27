use sqlx::postgres::PgPoolOptions;
use sqlx::{Error, PgPool};
use std::time;
use crate::cfg;

pub async fn get_db_pool() -> Result<PgPool, Error> {
    let config = cfg();
    let database_url = &config.database_url;

    PgPoolOptions::new()
        .max_connections(10)
        .acquire_timeout(time::Duration::from_secs(5))
        .connect(&database_url)
        .await
}
