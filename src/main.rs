mod quote;
use quote::*;
mod templates;
use templates::*;
use tower_http::services::ServeDir;

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

async fn db_connection() -> Cow<'static, str> {
    match env::var("DATABASE_URL") {
        Ok(url) => Cow::Owned(url), // Take ownership of the String
        Err(_) => Cow::Borrowed("no_url"), // Borrow static string
    }
}

async fn serve() -> Result<(), Box<dyn std::error::Error>> {
    // Set up the address:
    let localhost = "127.0.0.1";
    let port = "8000";
    let address = format!("{}:{}", localhost, port);

    let data_base_url: Cow<str> = db_connection().await;

    // Async database connection
    let (db_client, connection) = tokio_postgres::connect(&data_base_url, NoTls).await?;

    // Spawn the connection object to run in the background
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });

    // Async query execution
    let rows = db_client.query("SELECT * FROM quote.quotes", &[]).await?;

    // Process rows
    for row in rows {
        println!("{:?}", row);
    }

    // source: https://docs.rs/postgres/latest/postgres/struct.Client.html#method.connect
    //let mut db_client = Client::connect(&data_base_url, NoTls)?;
    /*let mut select_quotes = db_client.execute(
        "select * from quote.quotes;",
        &[],
    )?;*/

    //println!("{}", select_quotes);

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
