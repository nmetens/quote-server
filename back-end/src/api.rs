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
    OpenApiRouter::new()
        .routes(routes!(get_quote))
        .routes(routes!(get_tagged_quote))
        .routes(routes!(get_random_quote))
        .routes(routes!(add_quote))
        .routes(routes!(delete_quote))
        .routes(routes!(get_all_quotes))
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
    path = "/register",
    request_body(
        content = inline(authjwt::Registration),
        description = "Get an API key",
    ),
    responses(
        (status = 200, description = "JSON Web Token", body = authjwt::AuthBody),
        (status = 401, description = "Registration failed", body = authjwt::AuthError),
    )
)]
pub async fn register(
    State(appstate): State<SharedAppState>,
    Json(registration): Json<authjwt::Registration>,
) -> axum::response::Response {
    let appstate = appstate.read().await;
    match authjwt::make_jwt_token(&appstate, &registration) {
        Err(e) => e.into_response(),
        Ok(token) => (StatusCode::OK, token).into_response(),
    }
}

#[utoipa::path(
    post,
    path = "/add-quote",
    request_body = JsonQuote,
    responses(
        (status = 200, description = "Quote added successfully"),
        (status = 400, description = "Invalid input or duplicate ID"),
        (status = 500, description = "Server/database error")
    )
)]
pub async fn add_quote(
    State(app_state): State<Arc<RwLock<AppState>>>,
    axum::Json(json_quote): axum::Json<JsonQuote>,
) -> Result<impl axum::response::IntoResponse, StatusCode> {

    println!("Quote added: {:?}", json_quote);

    let app_reader = app_state.read().await;
    let db = &app_reader.db;

    let (quote, tags): (Quote, Vec<&str>) = {
        let (q, t_iter) = json_quote.to_quote();
        (q, t_iter.collect())
    };

    let mut tx = db.begin().await.map_err(|e| {
        log::error!("Failed to start transaction: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // Insert into quotes table
    sqlx::query!(
        "INSERT INTO quotes (id, quote, author) VALUES (?, ?, ?);",
        quote.id,
        quote.quote,
        quote.author
    )
    .execute(&mut *tx)
    .await
    .map_err(|e| {
        log::warn!("Insert quote failed: {}", e);
        StatusCode::BAD_REQUEST
    })?;

    // Insert associated tags
    for tag in tags {
        sqlx::query!(
            "INSERT INTO tags (quote_id, tag) VALUES (?, ?);",
            quote.id,
            tag
        )
        .execute(&mut *tx)
        .await
        .map_err(|e| {
            log::error!("Insert tag failed: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    }

    tx.commit().await.map_err(|e| {
        log::error!("Transaction commit failed: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    //Ok(StatusCode::OK)
    Ok(axum::Json(json_quote)) // Return the quote back.
}

#[utoipa::path(
    delete,
    path = "/delete-quote/{quote_id}",
    responses(
        (status = 200, description = "Quote deleted successfully"),
        (status = 404, description = "Quote not found"),
        (status = 500, description = "Database error")
    )
)]
pub async fn delete_quote(
    State(app_state): State<Arc<RwLock<AppState>>>,
    Path(quote_id): Path<String>,
) -> Result<impl IntoResponse, StatusCode> {
    let app_reader = app_state.read().await;
    let db = &app_reader.db;

    // Start transaction
    let mut tx = db.begin().await.map_err(|e| {
        log::error!("Failed to start transaction: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // First delete tags associated with the quote
    sqlx::query!("DELETE FROM tags WHERE quote_id = ?;", quote_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| {
            log::error!("Failed to delete tags: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    // Then delete the quote
    let result = sqlx::query!("DELETE FROM quotes WHERE id = ?;", quote_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| {
            log::error!("Failed to delete quote: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    if result.rows_affected() == 0 {
        return Err(StatusCode::NOT_FOUND);
    }

    tx.commit().await.map_err(|e| {
        log::error!("Transaction commit failed: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok((StatusCode::OK, format!("Quote {} deleted", quote_id)))
}

#[utoipa::path(
    get,
    path = "/all-quotes",
    responses(
        (status = 200, description = "Get all quotes", body = [JsonQuote]),
        (status = 500, description = "Database error")
    )
)]
pub async fn get_all_quotes(
    State(app_state): State<Arc<RwLock<AppState>>>,
) -> Result<impl IntoResponse, StatusCode> {
    let app_reader = app_state.read().await;
    let db = &app_reader.db;

    let rows = sqlx::query!("SELECT id, quote, author FROM quotes")
        .fetch_all(db)
        .await
        .map_err(|e| {
            log::error!("Failed to fetch quotes: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let mut quotes = Vec::new();

    for row in rows {
        let tags: Vec<String> = sqlx::query_scalar!(
            "SELECT tag FROM tags WHERE quote_id = ?;",
            row.id
        )
        .fetch_all(db)
        .await
        .map_err(|e| {
            log::error!("Failed to fetch tags for quote {}: {}", row.id, e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

        let quote = JsonQuote::new(
            Quote {
                id: row.id,
                quote: row.quote,
                author: row.author,
            },
            tags,
        );

        quotes.push(quote);
    }

    Ok((StatusCode::OK, axum::Json(quotes)))
}