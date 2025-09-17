use std::env;
use sqlx::{PgPool, Executor};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let database_url = env::var("DATABASE_URL")?;
    let pool = PgPool::connect(&database_url).await?;

    let schema_sql = include_str!("fix_schema.sql");

    pool.execute(schema_sql).await?;

    println!("Schema applied successfully!");

    Ok(())
}