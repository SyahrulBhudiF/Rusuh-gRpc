use dotenv::dotenv;
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;
use std::env;

pub async fn get_db_pool() -> Result<PgPool, sqlx::Error> {
    dotenv().ok();

    let database_url =
        env::var("DATABASE_URL").expect("DATABASE_URL must be set in .env file or environment");

    PgPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await
}
