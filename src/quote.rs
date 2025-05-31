use crate::*;

use std::collections::HashSet;
use std::ops::Deref;
use std::path::Path;

use axum::response::IntoResponse;
use axum::http::StatusCode;
use serde::Serialize;
use crate::QuoteError;

use serde::{Serialize, Deserialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct JsonQuote {
    pub id: i32,
    pub quote: String,
    pub author: String,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, utoipa::ToSchema)]

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct JsonQuote {
    id: i32,
    quote: String,
    author: String,
    tags: HashSet<String>,
}

#[derive(Clone)]

pub struct Quote {
    pub id: i32,
    pub quote: String,
    pub author: String,
}

pub fn read_quotes<P: AsRef<Path>>(quotes_path: P) -> 
    Result<Vec<JsonQuote>, QuoteError> {

    let f = std::fs::File::open(quotes_path.as_ref())?;
    let quotes = serde_json::from_reader(f)?;
    Ok(quotes)
}

impl JsonQuote {
    pub fn new(quote: Quote, tags: Vec<String>) -> Self {
        let tags = tags.into_iter().collect();
        Self {
            id: quote.id,
            quote: quote.quote,
            author: quote.author,
            tags,
        }
    }
    pub fn to_quote(&self) -> (Quote, impl Iterator<Item = &str>) {
        let quote = Quote {
            id: self.id.clone(),
            quote: self.quote.clone(),
            author: self.author.clone(),
        };
        let tags = self.tags.iter().map(String::deref);
        (quote, tags)
    }
}

impl axum::response::IntoResponse for &JsonQuote {
    fn into_response(self) -> axum::response::Response {
        (StatusCode::OK, axum::Json(&self)).into_response()
    }
}
pub async fn get(db: &SqlitePool, quote_id: &str) -> Result<(Quote, Vec<String>), sqlx::Error> {
    let quote = sqlx::query_as("select * from quotes where id = $1;")
        .bind(quote_id)
        .fetch_one(db)
        .await?;

    let tags: Vec<String> = sqlx::query_scalar!("SELECT tag FROM tags WHERE quote_id = $1;", quote_id)
        .fetch_all(db)
        .await?;

    Ok((quote, tags))
}

/
pub async fn get_quote_by_id(db: &SqlitePool, quote_id: &str) -> Result<(quote, Vec<String>), sqlx::Error> {
    let quote = sqlx::query_as!(Quote, "select * from quotes id = $1;", quote_id)
        .fetch_one(db)
        .await?;

    let tags: Vec<String> = sqlx::query_scalar!("select tag from tags where id = $1;", quote_id)
        .fetch_all(db)
        .await?;

    Ok((quote, tags))
}

pub async fn get_random(db: &SqlitePool) -> Result<String, sqlx::Error> {
    sqlx::query_scalar!("select id from quotes order by random() limit 1;")
        .fetch_one(db)
        .await
}