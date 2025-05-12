mod quote;
use quote::*;
mod templates;
use templates::*;
use tower_http::services::ServeDir;
use sqlx::sqlite::SqlitePoolOptions;

use std::fs;

use sqlx::{Row, SqlitePool, migrate::MigrateDatabase, sqlite};
use postgres::{Client, NoTls};
use std::borrow::Cow;

// For random quote:
use rand::Rng; // Source: https://rust-random.github.io/book/guide-values.html
use std::env; // For environment variable password to connect to the database.

use axum::{Router, routing::get, http::StatusCode, response::Html, response::IntoResponse};
use tokio::net::TcpListener;
use askama::Template;

struct AppState {
    db: SqlitePool,
    current_quote: Quote,
}

async fn get_quote() -> impl IntoResponse {
    let quotes = match read_quotes("static/famous_quotes.json") {
        Ok(quotes) => quotes,
        Err(err) => return (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("File read error: {}", err),
        ).into_response(),
    };

    // Generate a random number:
    let mut rng = rand::rng();
    let random_index: usize = rng.random_range(1..=100);

    //let quote = match quotes.first() {
    let quote = match quotes.get(random_index) {
        Some(q) => q.to_quote(),
        None => return (
            StatusCode::INTERNAL_SERVER_ERROR,
            "No quotes found".to_string(),
        ).into_response(),
    };

    //println!("{:?}", quote); // Quote Logging
    println!("Quote #{}: {} -{}", quote.id, quote.quote, quote.author); // Quote Logging

    let rendered = match IndexTemplate::quote(&quote).render() {
        Ok(html) => html,
        Err(err) => return (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Template error: {}", err),
        ).into_response(),
    };

    Html(rendered).into_response()
}

async fn serve() -> Result<(), Box<dyn std::error::Error>> {
    // Set up the address:
    let localhost = "127.0.0.1";
    let port = "8000";
    let address = format!("{}:{}", localhost, port);

    // Connect to the SQLite database: https://docs.rs/sqlx/latest/sqlx/pool/struct.PoolOptions.html
    let pool = SqlitePoolOptions::new()
        .connect("sqlite:quotes.db")
        .await?;

    // Read JSON file
    let file_content = fs::read_to_string("static/famous_quotes.json")?;
    // Get the quotes from the json file and put each quote in the vector:
    let quotes: Vec<Quote> = serde_json::from_str(&file_content)?;

    // Loop through the vector, putting all the quotes in the database: https://docs.rs/sqlx/latest/sqlx/fn.query.html
    /*for quote in quotes {
        sqlx::query("insert into quotes (id, quote, author) values (?, ?, ?);")
        .bind(&quote.id)
        .bind(&quote.quote)
        .bind(&quote.author)
        .execute(&pool)
        .await?;
    }*/

    let all_quotes = sqlx::query("select * from quotes;")
        .fetch_all(&pool).await?;

    for row in all_quotes {
        // Cargo: error[E0283]: type annotations needed 
        let id: i64 = row.get("id");
        let quote: &str = row.get("quote");
        let author: &str = row.get("author");

        println!("Quote #{}: {} -{}", id, quote, author);
    }

    // Set up the server:
    let listener = TcpListener::bind(address).await?;

    let app = Router::new()
        .route("/", get(get_quote))
        .nest_service("/static", ServeDir::new("static"));

    println!("Server running at http://{}:{}", localhost, port);

    axum::serve(listener, app).await?;

    Ok(())
}

#[tokio::main]
async fn main() {
    serve().await.expect("No famous quote found");
}
