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


// Route created: /tagged-quote
#[utoipa::path(
    get,
    path = "/tagged-quote",
    responses(
        (status = 200, description = "Get a quote by tags", body = [JsonQuote]),
        (status = 404, description = "No matching quotes"),
    )
)]
pub async fn get_tagged_quote(
    State(app_state): State<Arc<RwLock<AppState>>>, // Grab the shared app state.
    Json(tags): Json<Vec<String>>, // Grab the tags from the json.
) -> Result<response::Response, http::StatusCode> {

    log::info!("get tagged quote: {:?}", tags); // Print the result.

    // As before, get the app state and db:
    let app_reader = app_state.read().await; 
    let db = &app_reader.db;

    // The resulting quote from the tags by random:
    let quote_result = quote::get_tagged(db, tags.iter().map(String::as_ref)).await;

    match quote_result {
        Ok(Some(quote_id)) => get_quote_by_id(db, &quote_id).await, // Matching quote found and returned.

        Ok(None) => {
            log::warn!("quote tag fetch failed tagging"); // There was no tag like the one provided in the database.
            Err(http::StatusCode::NOT_FOUND)
        }

        Err(e) => {
            log::warn!("quote tag fetch failed: {}", e); // Error occured in db.
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