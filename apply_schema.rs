use sqlx::PgPool;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL environment variable must be set");

    let pool = PgPool::connect(&database_url).await?;

    // Read and execute the schema file
    let schema_sql = std::fs::read_to_string("complete_schema.sql")?;

    // Execute the complete schema
    println!("Applying complete database schema...");
    sqlx::raw_sql(&schema_sql).execute(&pool).await?;

    println!("Schema applied successfully!");

    pool.close().await;
    Ok(())
}