/// This file derives the OpenApi documentation for the ApiDoc struct.
/// There are three endpoints in this API: 
///     1) get_quote
///     2) get_tagged_quote
///     3) get_random_quote 
/// It uses utoipa for OpenAPI generation of the Swagger compatible docs.
/// There is a get_quote_by_id asychrnous method that is used in all api methods.
/// AppState is shared between enpoints to allow asynchronous visits.
/// All quotes are fetched using anychronous calls to the database.

use crate::*;
use crate::http::StatusCode;

#[derive(OpenApi)]
#[openapi(
    tags(
        (name = "quote-server", description = "Famous Quote API") // Swagger UI grouping.
    )
)]
pub struct ApiDoc; // Struct to create api references.

// Function that generates all the api endpoints with OpenApi:
pub fn router() -> OpenApiRouter<Arc<RwLock<AppState>>> {
    OpenApiRouter::new().routes(routes![
        get_quote,
        get_tagged_quote,
        get_random_quote,
        add_quote
    ])
}

// Method that queries the database looking for the quote_id that is passed in as an argument.
async fn get_quote_by_id(db: &SqlitePool, quote_id: &str) -> Result<response::Response, http::StatusCode> {

    let quote_result = quote::get(db, quote_id).await; // The resulting quote to return.

    match quote_result {
        Ok((quote, tags)) => Ok(JsonQuote::new(quote, tags).into_response()), // Wrap the quote and tags in JsonQuote struct.

        Err(e) => { // Quote was not found by the id provided. 404 not found displayed.
            log::warn!("quote fetch failed: {}", e);
            Err(http::StatusCode::NOT_FOUND)
        }
    }
}

// Route created: /quote/{quote_id}
// Go to the database, extract the correct quote by the id provided. Return error 404 if id not found.
#[utoipa::path(
    get,
    path = "/quote/{quote_id}",
    responses(
        (status = 200, description = "Get a quote by id", body = [JsonQuote]),
        (status = 404, description = "No matching quote"),
    )
)]
pub async fn get_quote(
    State(app_state): State<Arc<RwLock<AppState>>>, // Grab the app_state.
    Path(quote_id): Path<String>, // Grab the quote id from the url.
) -> Result<response::Response, http::StatusCode> {

    let app_reader = app_state.read().await; // Extract the app state.

    let db = &app_reader.db; // Grab the common database that is shared amoungst resources.

    get_quote_by_id(db, &quote_id).await // Call method and pass database and id to extract quote from database.
}
use axum::extract::Query;
use std::collections::HashMap;

#[utoipa::path(
    get,
    path = "/tagged-quote",
    params(
        ("tags" = Option<String>, Query, description = "Comma-separated tags")
    ),
    responses(
        (status = 200, description = "Get a quote by tags", body = [JsonQuote]),
        (status = 404, description = "No matching quotes"),
    )
)]
pub async fn get_tagged_quote(
    State(app_state): State<Arc<RwLock<AppState>>>,
    Query(tags_param): Query<HashMap<String, String>>,  // Use HashMap to extract 'tags'
) -> Result<response::Response, http::StatusCode> {
    let tags_string = tags_param.get("tags").cloned().unwrap_or_default();

    let tags: Vec<String> = tags_string
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

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

// Route created: /random-quote
// Grabs a random quote from the database.
#[utoipa::path(
    get,
    path = "/random-quote",
    responses(
        (status = 200, description = "Get a random quote", body = [JsonQuote]),
        (status = 404, description = "No quote"),
    )
)]
pub async fn get_random_quote(
    State(app_state): State<Arc<RwLock<AppState>>>, // Extract the shared app state.
) -> Result<response::Response, http::StatusCode> {

    let app_reader = app_state.read().await;

    let db = &app_reader.db; // Grab the database.

    let quote_result = quote::get_random(db).await; // Random quote selected from db.

    match quote_result {
        Ok(quote_id) => get_quote_by_id(db, &quote_id).await, // Found the quote.

        Err(e) => {
            log::warn!("get random quote failed: {}", e); // Error
            Err(http::StatusCode::NOT_FOUND)
        }
    }
}

#[utoipa::path(
    post,
    path = "/add-quote",
    request_body = Quote,
    responses(
        (status = 201, description = "Quote added successfully"),
        (status = 400, description = "Failed to add quote"),
    )
)]
pub async fn add_quote_handler(
    State(app_state): State<Arc<RwLock<AppState>>>,
    Json(new_quote): Json<JsonQuote>,
) -> Result<StatusCode, StatusCode> {
    let db = &app_state.read().await.db;

    match quote::add_quote(db, new_quote).await {
        Ok(_) => Ok(StatusCode::CREATED),
        Err(e) => {
            log::error!("Failed to add quote: {}", e);
            Err(StatusCode::BAD_REQUEST)
        }
    }
}