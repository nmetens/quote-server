mod quote;
use quote::*;
mod templates;
use templates::*;
use tower_http::services::ServeDir;

use tokio_postgres::Client;
use std::sync::Arc;

use tokio_postgres::NoTls;

// For random quote:
use rand::Rng; // Source: https://rust-random.github.io/book/guide-values.html
use std::env; // For environment variable password to connect to the database.

use axum::{Router, routing::get, http::StatusCode, response::Html, response::IntoResponse};
use tokio::net::TcpListener;
use askama::Template;

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

async fn db_connect() -> Result<Client, Box<dyn std::error::Error>> {
    // Connection credentials to the postgres database:
    let db_name = "spr25adb0047";
    let db_user = "spr25adb0047";
    let db_password = env::var("password")?; // Getting my local environment var for privacy
    let db_host = "dbclass.cs.pdx.edu";

    // Set up the database connection credential string:
    let conn_cred = format!(
        "host={} user={} password={} dbname={}",
        db_host, db_user, db_password, db_name 
    );
        
    // Connect to the postgres database
    let (client, connection) =
        tokio_postgres::connect(&conn_cred, NoTls).await?;

    // Spawn the background connection task
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("Database connection error: {}", e);
        }
    });

    Ok(client)
}

async fn serve() -> Result<(), Box<dyn std::error::Error>> {
    // Set up the address:
    let localhost = "127.0.0.1";
    let port = "8000";
    let address = format!("{}:{}", localhost, port);

    // Set up the server:
    let listener = TcpListener::bind(address).await?;

    // Connect to the databse:
    let client = db_connect().await?;
    let shared_client = Arc::new(client);
    println!("Connected to db");
 
    let app = Router::new()
        .route("/", get(get_quote))
        .nest_service("/static", ServeDir::new("static"))
        .with_state(shared_client);

    println!("Server running at http://{}:{}", localhost, port);

    axum::serve(listener, app).await?;

    Ok(())
}

#[tokio::main]
async fn main() {
    serve().await.expect("No famous quote found");
}
