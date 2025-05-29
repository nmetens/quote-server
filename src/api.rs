use crate::*;

#[derive(OpenApi)]
#[openapi(
    tags(
        (name = "quote-server", description = "Famous Quote API")
    )
)]
pub struct ApiDoc;
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