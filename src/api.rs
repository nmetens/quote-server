
use sqlx::SqlitePool;
use axum::response;
use axum::http;

use crate::quote;
use crate::JsonQuote;

pub async fn get_quote(db: &SqlitePool, quote_id: &str) -> Result<response::Response, http::StatusCode> {
    let quote_result = quote::get_quote_by_id(db, quote_id).await;

    match quote_result {
        Ok(quote) => {
            let json_quote = JsonQuote::new(quote.quote, quote.author);
            Ok(json_quote.into_response())
        }
        Err(e) => {
            println!("quote fetch failed: {}", e);
            Err(http::StatusCode::NOT_FOUND)
        }
    }
}

pub async fn get_random_quote() {

}
