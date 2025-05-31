mod quote;
mod api;

use api::*;
use quote::*;
mod templates;
use templates::*;
use tower_http::services::ServeDir;
use sqlx::sqlite::SqlitePoolOptions;
use sqlx::types::JsonValue::String;
use sqlx::Sqlite;

use std::fs;
use std::string::String as string;

use sqlx::Pool;
use sqlx::{Row, SqlitePool, migrate::MigrateDatabase, sqlite};
use std::borrow::Cow;

// For random quote:
use rand::Rng; // Source: https://rust-random.github.io/book/guide-values.html
use std::env; // For environment variable password to connect to the database.

use axum::routing::get;
use axum::{Router, routing, http::StatusCode, response::Html, response::IntoResponse};
use tokio::net::TcpListener;
use askama::Template;

use utoipa::{OpenApi, ToSchema};
use utoipa_axum::{router::OpenApiRouter, routes};
use utoipa_rapidoc::RapiDoc;
use utoipa_redoc::{Redoc, Servable};
use utoipa_swagger_ui::SwaggerUi;

use clap::Parser;
extern crate fastrand;
use serde::{Serialize, Deserialize};
use tokio::{net, sync::RwLock};
use tower_http::{services, trace};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use std::sync::Arc;

struct AppState {
    db: SqlitePool,
    current_quote: Quote,
}

async fn rand_quote() -> (usize, Quote) {
    let starting_quote: Quote = Quote {
        id: 101,
        quote: "Yesterday is history, tomorrow is a mystery, but today is a gift. That's why it's called the present".to_string(),
        author: "-Oogway".to_string(),
    };

    let quotes = match read_quotes("static/famous_quotes.json") {
        Ok(quotes) => quotes,
        _ => return (111, starting_quote),
    };

    // Generate a random number:
    let mut rng = rand::rng();
    let random_index: usize = rng.random_range(1..=100);

    //let quote = match quotes.first() {
    let quote = match quotes.get(random_index) {
        Some(q) => q.to_quote(),
        None => return (111, starting_quote),
    };

    (random_index, quote)
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

async fn json_to_db(pool: Pool<sqlx::Sqlite>) {
    // Read JSON file
    let file_content = fs::read_to_string("static/famous_quotes.json").expect("No json file.");
    // Get the quotes from the json file and put each quote in the vector:
    let quotes: Vec<Quote> = serde_json::from_str(&file_content).expect("No json file.");

    // Loop through the vector, putting all the quotes in the database
    // source: https://docs.rs/sqlx/latest/sqlx/fn.query.html
    for quote in quotes {
        let _ = sqlx::query("insert into quotes (id, quote, author) values (?, ?, ?);")
        .bind(&quote.id)
        .bind(&quote.quote)
        .bind(&quote.author)
        .execute(&pool)
        .await;
    }
}

async fn get_quotes(pool: Pool<sqlx::Sqlite>) -> Result<Vec<Quote>, sqlx::Error> {
    let rows = sqlx::query("SELECT * FROM quotes;")
        .fetch_all(&pool)
        .await?;

    let mut quotes: Vec<Quote> = Vec::new();

    for row in rows {
        let quote = Quote {
            id: row.get("id"),
            quote: row.get("quote"),
            author: row.get("author"),
        };
        quotes.push(quote);
    }
    Ok(quotes)
}

async fn serve() -> Result<(), Box<dyn std::error::Error>> {
    // Set up the address:
    let localhost = "127.0.0.1";
    let port = "8000";
    let address = format!("{}:{}", localhost, port);

    // Connect to the SQLite database: https://docs.rs/sqlx/latest/sqlx/pool/struct.PoolOptions.html
    let pool: Pool<sqlx::Sqlite> = SqlitePoolOptions::new()
        .connect("sqlite:quotes.db")
        .await?;

    //let state = AppState { pool, };

    // Set up the server:
    let listener = TcpListener::bind(address).await?;

    /*let apis: Router<Pool<Sqlite>> = Router::new()
        .route("/quote", get(api::get_all_quotes_api))
        .route("/quote/{quote_id}", get(api::get_quote))
        //.route("/quote/{quote_id}", get(api::get_quote_by_id))
        .route("/random-quote", get(api::get_random_quote));*/

    let (api_router, api) = OpenApiRouter::with_openapi(api::ApiDoc::openapi())
        .nest("/api/v1", api::router())
        .split_for_parts();

    let swagger_ui = SwaggerUi::new("/swagger-ui")
        .url("/api-docs/openapi.json", api.clone());
    let redoc_ui = Redoc::with_url("/redoc", api);
    let rapidoc_ui = RapiDoc::new("/api-docs/openapi.json").path("/rapidoc");

    let app = Router::new()
        .route("/", get(get_quote))
        .nest_service("/static", ServeDir::new("static"))
        .nest("/api", apis)
        .merge(swagger_ui)
        .merge(redoc_ui)
        .merge(rapidoc_ui)
        .merge(api_router)
        .with_state(pool.clone());

    println!("Server running at http://{}:{}", localhost, port);

    axum::serve(listener, app).await?;

    Ok(())
}

#[tokio::main]
async fn main() {
    serve().await.expect("No famous quote found");
}
