use std::path::Path;
use serde::Deserialize;
use sqlx::SqlitePool;
use sqlx::Row;
use axum::Json;

use axum::response::IntoResponse;
use axum::http::StatusCode;
use serde::Serialize;


#[derive(Clone, Serialize, Deserialize)]
pub struct JsonQuote {
    pub id: i32,
    pub quote: String,
    pub author: String,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, utoipa::ToSchema)]
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
    pub fn to_quote(&self) -> Quote {
        Quote {
            id: self.id,
            quote: self.quote.clone(),
            author: self.author.clone(),
        }
    }
}

impl JsonQuote {
    pub fn new(id: i32, quote: String, author: String) -> Self {
        JsonQuote { id, quote, author }
    }
}

/*impl axum::response::IntoResponse for &JsonQuote {
    fn into_response(self) -> axum::response::Response {
        (StatusCode::OK, axum::Json(self)).into_response()
    }
}*/
/*impl IntoResponse for &JsonQuote {
    fn into_response(self) -> axum::response::Response {
        (StatusCode::OK, Json(self)).into_response()
    }
}*/

pub async fn get_quote_by_id(db: &SqlitePool, quote_id: &str) -> Result<Quote, sqlx::Error> {
    let quote = sqlx::query_as::<_, Quote>("SELECT * FROM quotes WHERE id = ?;")
        .bind(&quote_id)
        .fetch_one(db)
        .await?;

    Ok(quote)
}
