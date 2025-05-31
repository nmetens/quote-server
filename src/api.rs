use crate::*;
use rand::thread_rng;
use sqlx::SqlitePool;
use axum::response::{Response, IntoResponse};
use axum::http::StatusCode;

use axum::Json;
use crate::quote;
use crate::JsonQuote;
use axum::extract::Path;

pub async fn get_quote_by_id(
    Path(joke_id): Path<i32>,
) -> impl IntoResponse {
    let quotes = match read_quotes("static/famous_quotes.json") {
        Ok(quotes) => quotes,
        Err(err) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to read quotes: {}", err),
            ).into_response();
        }
    };

    match quotes.iter().find(|q| q.id == joke_id) {
        Some(quote) => Json(quote.clone()).into_response(),
        None => (
            StatusCode::NOT_FOUND,
            format!("No quote found with ID {}", joke_id),
        ).into_response(),
    }
}

pub async fn get_random_quote() -> impl IntoResponse {
    let quotes = match read_quotes("static/famous_quotes.json") {
        Ok(quotes) => quotes,
        Err(err) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to read quotes: {}", err),
            ).into_response();
        }
    };

    if quotes.is_empty() {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            "No quotes available".to_string(),
        ).into_response();
    }

    let mut rng = thread_rng();
    let index = rng.gen_range(0..quotes.len());
    let quote = &quotes[index];

    Json(quote).into_response()
}
/*pub async fn get_quote(db: &SqlitePool, quote_id: &str) -> Result<Response, StatusCode> {
    let quote_result = quote::get_quote_by_id(db, quote_id).await;

    match quote_result {
        Ok(quote) => {
            let json_quote = JsonQuote::new(quote.id, quote.quote, quote.author);
            Ok((StatusCode::OK, Json(json_quote)).into_response())
        }
        Err(e) => {
            println!("quote fetch failed: {}", e);
            Err(StatusCode::NOT_FOUND)
        }
    }
}*/

