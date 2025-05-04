use sqlx::postgres::PgPoolOptions;
use sqlx::{Error, PgPool};
use std::{env, time};

pub async fn get_db_pool() -> Result<PgPool, Error> {
    let database_url =
        env::var("DATABASE_URL").expect("DATABASE_URL must be set in .env file or environment");

    PgPoolOptions::new()
        .max_connections(10)
        .acquire_timeout(time::Duration::from_secs(5))
        .connect(&database_url)
        .await
}
