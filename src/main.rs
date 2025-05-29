mod quote;
mod error;
mod api;
mod web;

use api::*;
use quote::*;
use error::*;
mod templates;
use templates::*;
use tower_http::services::ServeDir;
use utoipa::{OpenApi, ToSchema};
use std::sync::Arc;

use axum::{
    self,
    extract::{Path, Query, State, Json},
    http,
    response::{self, IntoResponse},
    routing,
};

use clap::Parser;
use serde::{Deserialize};
use sqlx::{SqlitePool, migrate::MigrateDatabase, sqlite};
use tokio::{sync::RwLock};
use sqlx::Pool;
use std::borrow::Cow;

// For random quote:
use axum::routing::get;
use axum::{Router, http::StatusCode, response::IntoResponse};
use tokio::net::TcpListener;

#[derive(Parser)]
struct Args {
    #[arg(short, long, name = "init-from")]
    init_from: Option<std::path::PathBuf>,
    #[arg(short, long, name = "db-uri")]
    db_uri: Option<String>,
}

struct AppState {
    db: SqlitePool,
    current_quote: Quote,
}

fn get_db_uri(db_uri: Option<&str>) -> Cow<str> {
    if let Some(db_uri) = db_uri {
        db_uri.into()
    } else if let Ok(db_uri) = std::env::var("DATABASE_URL") {
        db_uri.into()
    } else {
        "sqlite://db/quotes.db".into()
    }
}
fn extract_db_dir(db_uri: &str) -> Result<&str, QuoteError> {
    if db_uri.starts_with("sqlite://") && db_uri.ends_with(".db") {
        let start = db_uri.find(':').unwrap() + 3;
        let mut path = &db_uri[start..];
        if let Some(end) = path.rfind('/') {
            path = &path[..end];
        } else {
            path = "";
        }
        Ok(path)
    } else {
        Err(QuoteError::InvalidDbUri(db_uri.to_string()))
    }
}

/*
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
*/
/*async fn get_quote() -> impl IntoResponse {
    
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
*/
/*async fn json_to_db(pool: Pool<sqlx::Sqlite>) {
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
*/
/*async fn get_quotes(pool: Pool<sqlx::Sqlite>) -> Result<Vec<Quote>, sqlx::Error> {
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
}*/

async fn serve() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let db_uri: Cow<'_, str> = get_db_uri(args.db_uri.as_deref());
    if !sqlite::Sqlite::database_exists(&db_uri).await? {
        let db_dir = extract_db_dir(&db_uri)?;
        std::fs::create_dir_all(db_dir)?;
        sqlite::Sqlite::create_database(&db_uri).await?
    }
    let db: Pool<sqlx::Sqlite> = SqlitePool::connect(&db_uri).await?;
    sqlx::migrate!().run(&db).await?;

    println!("{:?}", db);
    println!("Resolved DB URI: {}", db_uri);
    if let Some(path) = args.init_from {
        let quotes = read_quotes(path)?;
        'next_quote: for qu in quotes {
            let mut qtx = db.begin().await?;
            let (q, ts) = qu.to_quote();
            let quote_insert = sqlx::query(
                "insert into quotes (id, quote, author) values($1, $2, $3);")
                .bind(q.id)
                .bind(q.quote)
                .bind(q.author)
                .execute(&mut *qtx)
                .await;
            if let Err(e) = quote_insert {
                eprintln!("error: quote insert: {}: {}", q.id, e);
                qtx.rollback().await?;
                continue;
            };
            for t in ts {
                let tag_insert =
                    sqlx::query("insert into tags (quote_id, tag) values ($1, $2);")
                        .bind(q.id)
                        .bind(t)
                        .execute(&mut *qtx)
                        .await;
                if let Err(e) = tag_insert {
                    eprintln!("error: tag insert: {} {}: {}", q.id, t, e);
                    qtx.rollback().await?;
                    continue 'next_quote;
                };
            }
            qtx.commit().await?;
        }
        return Ok(());
    }
    
    let current_quote = Quote {
        id: 101,
        quote: "Yesterday is history, tomorrow is a mystery, and today is a gift, that's why it's called the present.".to_string(),
        author: "Turtle".to_string(),
    };
    let app_state = AppState { db, current_quote};
    let state = Arc::new(RwLock::new(app_state));

    // Set up the address:
    let localhost = "127.0.0.1";
    let port = "8000";
    let address = format!("{}:{}", localhost, port);

    // Set up the server:
    let listener = TcpListener::bind(address).await?;

    /* 
    // Connect to the SQLite database: https://docs.rs/sqlx/latest/sqlx/pool/struct.PoolOptions.html
    let pool: Pool<sqlx::Sqlite> = SqlitePoolOptions::new()
        .connect("sqlite:quotes.db")
        .await?;

    let (api_router, api) = OpenApiRouter::with_openapi(api::ApiDoc::openapi())
        .nest("/api/v1", api::router())
        .split_for_parts();
    */
    let app = Router::new()
        .route("/", get(web::get_quote))
        .nest_service("/static", ServeDir::new("static"));

    println!("Server running at http://{}:{}", localhost, port);

    axum::serve(listener, app).await?;

    Ok(())
}

#[tokio::main]
async fn main() {
    serve().await.expect("No famous quote found");
}
