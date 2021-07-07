#![allow(unused)] // silence unused warnings

use sqlx::postgres::{PgPoolOptions, PgRow};
use sqlx::{FromRow, Row};

#[derive(Debug, FromRow)]
struct Ticket {
    id: i64,
    name: String,
}

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    // 1) Create a connection pool
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect("postgres://postgres:mysecretpassword@localhost/postgres")
        .await.expect("No connection available");

    // 2) Create table if not exist yet
    sqlx::query(
        r#"
CREATE TABLE IF NOT EXISTS ticket (
    id bigserial,
    name text
);"#,
    )
    .execute(&pool)
    .await?;

    // 3) Insert a new ticket
    let row: (i64,) = sqlx::query_as("insert into ticket (name) values ($1) returning id")
        .bind("a new ticket")
        .fetch_one(&pool)
        .await?;

    // 4) Select all tickets    
    let rows = sqlx::query("SELECT * FROM ticket")
        .fetch_all(&pool)
        .await?;

    let str_result = rows
        .iter()
        .map(|r| format!("{} - {}", r.get::<i64, _>("id"), r.get::<String, _>("name")))
        .collect::<Vec<String>>()
        .join(", ");
    
    println!("\n=== select tickets with PgRows:\n{}", str_result);

    // 5) Select query with map() (Build ticket manually)
    let select_query = sqlx::query("SELECT id, name FROM ticket");
    let tickets: Vec<Ticket> = select_query
        .map(|row: PgRow| Ticket {
            id: row.get("id"),
            name: row.get("name")
        })
        .fetch_all(&pool)
        .await?;
    println!("\n=== select tickets with query.map...:\n{:?}", tickets);

    // 6) Select query_as (using derive FromRow)
    let select_query = sqlx::query_as::<_, Ticket>("SELECT id, name FROM ticket");
    let tickets: Vec<Ticket> = select_query.fetch_all(&pool).await?;
    println!("\n=== select tickets with query.map...: \n{:?}", tickets);

    Ok(())
}
