mod error;
mod quote;
mod templates;
mod web;
mod api;

use error::*;
use quote::*;
use templates::*;

extern crate log;
extern crate mime;

use axum::{
    self,
    extract::{Path, Query, State, Json},
    http,
    response::{self, IntoResponse},
    routing,
};

use clap::Parser;
use serde::{Serialize, Deserialize};
use sqlx::{Row, SqlitePool, migrate::MigrateDatabase, sqlite};
use tokio::{net, sync::RwLock};
use tower_http::{services, trace};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use utoipa::{OpenApi, ToSchema};
use utoipa_axum::{router::OpenApiRouter, routes};
use utoipa_rapidoc::RapiDoc;
use utoipa_redoc::{Redoc, Servable};
use utoipa_swagger_ui::SwaggerUi;

use std::borrow::Cow;
use std::sync::Arc;
use tower_http::services::ServeDir;

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
    let args = Args::parse(); // Parse the cli arguments and flags.

    // Get the database uri.
    let db_uri: Cow<'_, str> = get_db_uri(args.db_uri.as_deref());

    // Check that the database uri exists. Otherwise, create it:
    if !sqlite::Sqlite::database_exists(&db_uri).await? {
        let db_dir = extract_db_dir(&db_uri)?;
        std::fs::create_dir_all(db_dir)?;
        sqlite::Sqlite::create_database(&db_uri).await?
    }

    // Connect to the database through the uri:
    let db = SqlitePool::connect(&db_uri).await?;
    sqlx::migrate!().run(&db).await?; // Run the migrations in the migrations dir.

    // If a path is given with the '--init_form' command, then load the quotes
    // from that file into the database:
    if let Some(path) = args.init_from {
        let quotes = read_quotes(path)?;
        'next_quote: for qu in quotes {
            let mut qtx = db.begin().await?;
            let (q, ts) = qu.to_quote();

            let quote_insert = sqlx::query!(
                "insert into quotes (id, quote, author) values($1, $2, $3);",
                q.id,
                q.quote,
                q.author,
            )
            .execute(&mut *qtx)
            .await;

            if let Err(e) = quote_insert {
                eprintln!("error: quote insert: {}: {}", q.id.clone(), e);
                qtx.rollback().await?;
                continue;
            };

            for t in ts {
                let tag_insert =
                    sqlx::query!("insert into tags (quote_id, tag) values ($1, $2);",
                        q.id,
                        t,
                    )
                    .execute(&mut *qtx)
                    .await;

                if let Err(e) = tag_insert {
                    eprintln!("error: tag insert: {} {}: {}", q.id.clone(), t, e);
                    qtx.rollback().await?;
                    continue 'next_quote;
                };
            }
            qtx.commit().await?;
        }
        return Ok(());
    }
    
    // The default quote displayed on the page before any other quote is displayed:
    let current_quote = Quote {
        id: "101".to_string(),
        quote: "Yesterday is history, tomorrow is a mystery, and today is a gift, that's why it's called the present.".to_string(),
        author: "Turtle".to_string(),
    };

    // Initialize the app state object with the db pool and the initial quote.
    let app_state = AppState { db, current_quote};

    // Make the state sharable for async reading and writing.
    let state = Arc::new(RwLock::new(app_state));

     // Initialize logging and tracing:
     tracing_subscriber::registry()
     .with(
         tracing_subscriber::EnvFilter::try_from_default_env()
             .unwrap_or_else(|_| "quote-server=debug,info".into()),
     )
     .with(tracing_subscriber::fmt::layer())
     .init();

    // https://carlosmv.hashnode.dev/adding-logging-and-tracing-to-an-axum-app-rust
    let trace_layer = trace::TraceLayer::new_for_http()
        .make_span_with(trace::DefaultMakeSpan::new().level(tracing::Level::INFO))
        .on_response(trace::DefaultOnResponse::new().level(tracing::Level::INFO));

    // Get log:
    let cors = tower_http::cors::CorsLayer::new()
        .allow_methods([http::Method::GET])
        .allow_origin(tower_http::cors::Any);

    // Page not found:
    async fn handler_404() -> axum::response::Response {
        (http::StatusCode::NOT_FOUND, "404 Not Found").into_response()
    }

    // API available under url ../api/v1.
    // Serve the api endpoints there.
    let (api_router, api) = OpenApiRouter::with_openapi(api::ApiDoc::openapi())
        .nest("/api/v1", api::router())
        .split_for_parts();

    let swagger_ui = SwaggerUi::new("/swagger-ui")
        .url("/api-docs/openapi.json", api.clone());
    let redoc_ui = Redoc::with_url("/redoc", api);
    let rapidoc_ui = RapiDoc::new("/api-docs/openapi.json").path("/rapidoc");

    // Build the app router. Connections to the styling, favicon, static files, etc.
    let app = axum::Router::new()
        .route("/", routing::get(web::get_quote))
        .route_service(
            "/quote.css",
            services::ServeFile::new_with_mime("assets/static/quote.css", &mime::TEXT_CSS_UTF_8),
        )
        .route_service(
            "/favicon.ico",
            services::ServeFile::new_with_mime("assets/static/heart.png", &mime::IMAGE_PNG),
        )
        .nest_service("/static", ServeDir::new("assets/static"))
        .merge(swagger_ui)
        .merge(redoc_ui)
        .merge(rapidoc_ui)
        .merge(api_router)
        .fallback(handler_404)
        .layer(cors)
        .layer(trace_layer)
        .with_state(state);

    // Set up the address information:
    let localhost = "127.0.0.1";
    let address = format!("{}:{}", localhost, args.port);
    println!("Server running at http://{}", address);

    // Set up the tcp listener with the args.port default to 8000:
    let listener = net::TcpListener::bind(&format!("{}:{}", localhost, args.port)).await?;
    println!("Listening on {}:{}", localhost, args.port);

    // Start the server and listen on the correct port:
    axum::serve(listener, app).await?;

    Ok(())
}

#[tokio::main]
async fn main() {
    serve().await.expect("No famous quote found");
}
