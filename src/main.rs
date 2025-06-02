mod quote;
mod error;
mod api;
mod web;
mod templates;

use api::*;
use quote::*;
use error::*;
use templates::*;

use sqlx::sqlite::SqlitePoolOptions;
use sqlx::types::JsonValue::String;
use sqlx::Sqlite;
use tower_http::services::ServeDir;

use std::fs;
use std::string::String as string;

use sqlx::Pool;
use sqlx::{Row, SqlitePool, migrate::MigrateDatabase, sqlite};

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

// Create the Args struct for the command line interface.
// Useful for parsing flags such as '--init_from', '--db_uri', '--port'
// and sets their defaults here:
#[derive(Parser)] // Parses the cli argumnets or flags.
struct Args {
    #[arg(short, long, name = "init-from")]
    init_from: Option<std::path::PathBuf>, // Optional path to initialize database path.
    #[arg(short, long, name = "db-uri")]
    db_uri: Option<String>, // Optional database uri.
    #[arg(short, long, default_value = "8000")]
    port: u16, // Default port of 8000
}

// The struct that holds the current quote and the database connection pool.
// This is a shared resource for the application. It is protected view the
// Rwlock type.
struct AppState {
    db: SqlitePool,
    current_quote: Quote, // Thec current quote for the initial display.
}

// Method to get the db_uri from the exported env variable.
// Can be set by:
//  1) $ export DATABASE_URL=sqlite://db/quotes.db
//  2) .env:
//      DATABASE_URL=sqlite://db/quotes.db
//  3) Or, if neither is used, the default is 'sqlite://db/quotes.db'.
fn get_db_uri(db_uri: Option<&str>) -> Cow<str> {
    if let Some(db_uri) = db_uri {
        db_uri.into()
    } else if let Ok(db_uri) = std::env::var("DATABASE_URL") {
        db_uri.into()
    } else {
        "sqlite://db/quotes.db".into()
    }
}

// Function that searches for the quotes.db file by its ending
// file type, and where it is located in the file system.
// Checks that the db_uri starts with the 'sqlite://' syntax and 
// exists before establishing a connection:
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
