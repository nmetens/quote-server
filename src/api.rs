use crate::*;


use rand::thread_rng;
use sqlx::SqlitePool;
use axum::response::{Response, IntoResponse};
use axum::http::StatusCode;

use axum::Json;
use crate::quote;
use crate::JsonQuote;
use axum::extract::Path;
use axum::extract::State;

use crate::*;

#[derive(OpenApi)]
#[openapi(
    tags(
        (name = "quote-server", description = "Famous Quote API")
    )
)]
pub struct ApiDoc;


pub fn router() -> OpenApiRouter<Arc<RwLock<AppState>>> {
    OpenApiRouter::new()
        .routes(routes!(get_quote))
        .routes(routes!(get_tagged_quote))
        .routes(routes!(get_random_quote))
}

/* 
pub fn router() -> OpenApiRouter<Arc<RwLock<AppState>>> {
    OpenApiRouter::new()
        .routes(routes!(get_quote))
        .routes(routes!(get_tagged_quote))
        .routes(routes!(get_random_quote))
}

async fn get_quote_by_id(db: &SqlitePool, quote_id: &str) -> Result<response::Response, http::StatusCode> {
    let quote_result = quote::get(db, quote_id).await;
    match quote_result {
        Ok((quote, tags)) => Ok(JsonQuote::new(quote, tags).into_response()),
        Err(e) => {
            log::warn!("quote fetch failed: {}", e);
            Err(http::StatusCode::NOT_FOUND)
        }
    }
}

#[utoipa::path(
    get,
    path = "/quote/{quote_id}",
    responses(
        (status = 200, description = "Get a quote by id", body = [JsonQuote]),
        (status = 404, description = "No matching quote"),
    )
)]
pub async fn get_quote(
    State(app_state): State<Arc<RwLock<AppState>>>,
    Path(quote_id): Path<String>,
) -> Result<response::Response, http::StatusCode> {
    let app_reader = app_state.read().await;
    let db = &app_reader.db;
    get_quote_by_id(db, &quote_id).await
}

#[utoipa::path(
    get,
    path = "/tagged-quote",
    responses(
        (status = 200, description = "Get a quote by tags", body = [JsonQuote]),
        (status = 404, description = "No matching quotes"),
    )
)]
pub async fn get_tagged_quote(
    State(app_state): State<Arc<RwLock<AppState>>>,
    Json(tags): Json<Vec<String>>,
) -> Result<response::Response, http::StatusCode> {
    log::info!("get tagged quote: {:?}", tags);
    let app_reader = app_state.read().await;
    let db = &app_reader.db;
    let quote_result = quote::get_tagged(db, tags.iter().map(String::as_ref)).await;
    match quote_result {
        Ok(Some(quote_id)) => get_quote_by_id(db, &quote_id).await,
        Ok(None) => {
            log::warn!("quote tag fetch failed tagging");
            Err(http::StatusCode::NOT_FOUND)
        }
        Err(e) => {
            log::warn!("quote tag fetch failed: {}", e);
        Err(e) => {
            log::warn!("quote fetch failed: {}", e);
            Err(http::StatusCode::NOT_FOUND)
        }
    }
}

#[utoipa::path(
    get,
    path = "/random-quote",
    responses(
        (status = 200, description = "Get a random quote", body = [JsonQuote]),
        (status = 404, description = "No quote"),
    )
)]
pub async fn get_random_quote(
    State(app_state): State<Arc<RwLock<AppState>>>,
) -> Result<response::Response, http::StatusCode> {
    let app_reader = app_state.read().await;
    let db = &app_reader.db;
    let quote_result = quote::get_random(db).await;
    match quote_result {
        Ok(quote_id) => get_quote_by_id(db, &quote_id).await,
        Err(e) => {
            log::warn!("get random quote failed: {}", e);
            Err(http::StatusCode::NOT_FOUND)
        }
    }
}

/* Below code implementation is non-Utopia: */
/*pub async fn get_all_quotes_api(
    State(pool): State<SqlitePool>,
) -> impl IntoResponse {
    match get_quotes(pool).await {
        Ok(quotes) => Json(quotes).into_response(),
        Err(err) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            format!("Database error: {}", err),
        ).into_response(),
    }
}

pub async fn get_quote_by_id(
    Path(quote_id): Path<i32>,
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

    match quotes.iter().find(|q| q.id == quote_id) {
        Some(quote) => Json(quote.clone()).into_response(),
        None => (
            StatusCode::NOT_FOUND,
            format!("No quote found with ID {}", quote_id),
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

pub async fn get_quote(
    db: &SqlitePool,
    quote_id: &str,
) -> impl IntoResponse {
    match quote::get_quote_by_id(db, quote_id).await {
        Ok(quote) => {
            let json_quote = JsonQuote::new(quote.id, quote.quote, quote.author);
            (StatusCode::OK, Json(json_quote)).into_response()
        }
        Err(e) => {
            eprintln!("quote fetch failed: {}", e);
            (StatusCode::NOT_FOUND, "Quote not found").into_response()
        }
    }
}*/

    path = "/quote/{quote_id}",
    responses(
        (status = 200, description = "Get a quote by id", body = [JsonQuote]),
        (status = 404, description = "No matching quote"),
    )
)]
pub async fn get_quote(
    State(app_state): State<Arc<RwLock<AppState>>>,
    Path(quote_id): Path<String>,
) -> Result<response::Response, http::StatusCode> {
    let app_reader = app_state.read().await;
    let db = &app_reader.db;
    get_quote_by_id(db, &quote_id).await
}

#[utoipa::path(
    get,
    path = "/tagged-quote",
    responses(
        (status = 200, description = "Get a quote by tags", body = [JsonQuote]),
        (status = 404, description = "No matching quotes"),
    )
)]
pub async fn get_tagged_quote(
    State(app_state): State<Arc<RwLock<AppState>>>,
    Json(tags): Json<Vec<String>>,
) -> Result<response::Response, http::StatusCode> {
    log::info!("get tagged quote: {:?}", tags);
    let app_reader = app_state.read().await;
    let db = &app_reader.db;
    let quote_result = quote::get_tagged(db, tags.iter().map(String::as_ref)).await;
    match quote_result {
        Ok(Some(quote_id)) => get_quote_by_id(db, &quote_id).await,
        Ok(None) => {
            log::warn!("quote tag fetch failed tagging");
            Err(http::StatusCode::NOT_FOUND)
        }
        Err(e) => {
            log::warn!("quote tag fetch failed: {}", e);
            Err(http::StatusCode::NOT_FOUND)
        }
    }
}

#[utoipa::path(
    get,
    path = "/random-quote",
    responses(
        (status = 200, description = "Get a random quote", body = [JsonQuote]),
        (status = 404, description = "No quote"),
    )
)]
pub async fn get_random_quote(
    State(app_state): State<Arc<RwLock<AppState>>>,
) -> Result<response::Response, http::StatusCode> {
    let app_reader = app_state.read().await;
    let db = &app_reader.db;
    let quote_result = quote::get_random(db).await;
    match quote_result {
        Ok(quote_id) => get_quote_by_id(db, &quote_id).await,
        Err(e) => {
            log::warn!("get random quote failed: {}", e);
            Err(http::StatusCode::NOT_FOUND)
        }
    }
}*/
