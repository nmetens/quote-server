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

async fn serve() -> Result<(), Box<dyn std::error::Error>> {
    // Set up the address:
    let localhost = "127.0.0.1";
    let port = "8000";
    let address = format!("{}:{}", localhost, port);

    // Set up the server:
    let listener = TcpListener::bind(address).await?;

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
