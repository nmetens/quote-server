use crate::*;
use std::path::Path;
use serde::{Serialize, Deserialize};
use std::collections::HashSet;
use std::ops::Deref;
use sqlx::{SqlitePool, Row};
use axum::{response::IntoResponse, http::StatusCode, Json};

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct JsonQuote {
    id: i32,
    quote: String,
    author: String,
    //tags: HashSet<String>,
}

#[derive(Clone)]
pub struct Quote {
    pub id: i32,
    pub quote: String,
    pub author: String,
}

pub fn read_quotes<P: AsRef<Path>>(quotes_path: P) -> 
    Result<Vec<JsonQuote>, Box<dyn std::error::Error>> {

    let f = std::fs::File::open(quotes_path.as_ref())?;
    let quotes = serde_json::from_reader(f)?;
    Ok(quotes)
}

impl JsonQuote {
    pub fn new(quote: Quote/* , tags: Vec<String>*/) -> Self {
        //let tags = tags.into_iter().collect();
        Self {
            id: quote.id,
            quote: quote.quote,
            author: quote.author,
            //tags,
        }
    }
    pub fn to_quote(&self) -> Quote {
        Quote {
            id: self.id.clone(),
            quote: self.quote.clone(),
            author: self.author.clone(),
        }
        //let tags = self.tags.iter().map(String::deref);
        //(quote, tags)
    }
}

impl axum::response::IntoResponse for &JsonQuote {
    fn into_response(self) -> axum::response::Response {
        (StatusCode::OK, axum::Json(&self)).into_response()
    }
}

pub async fn get_quote_by_id(db: &SqlitePool, quote_id: &str) -> Result<(Quote/* , Vec<String>*/), sqlx::Error> {
    let quote = sqlx::query_as!(Quote, "select * from quotes id = $1;", quote_id)
        .fetch_one(db)
        .await?;

    Ok(quote)
}