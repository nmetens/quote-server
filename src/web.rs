use crate::*;

#[derive(Deserialize)]
pub struct GetQuoteParams {
    id: Option<String>,
    tags: Option<String>,
}

pub async fn get_quote(
    State(app_state): State<Arc<RwLock<AppState>>>,
    Query(params): Query<GetQuoteParams>,
) -> Result<response::Response, http::StatusCode> {
    let mut app_writer = app_state.write().await;
    let db = app_writer.db.clone();

    // Specified.
    if let GetQuoteParams { id: Some(id), .. } = params {
        let quote_result = quote::get(&db, &id).await;
        let result = match quote_result {
            Ok((quote, tags)) => {
                let tag_string = tags.join(", ");

                app_writer.current_quote = quote.clone();
                let quote = IndexTemplate::new(quote.clone(), tag_string);
                Ok(response::Html(quote.to_string()).into_response())
            }
            Err(e) => {
                log::warn!("quote fetch failed: {}", e);
                Err(http::StatusCode::NOT_FOUND)
            }
        };
        return result;
    }

    
    if let GetQuoteParams { tags: Some(tags), .. } = params {
        log::info!("quote tags: {}", tags);

        let mut tags_string = String::new();
        for c in tags.chars() {
            if c.is_alphabetic() || c == ',' {
                let cl: String = c.to_lowercase().collect();
                tags_string.push_str(&cl);
            }
        }

        let quote_result = quote::get_tagged(&db, tags_string.split(',')).await;
        match quote_result {
            Ok(Some(id)) => {
                let uri = format!("/?id={}", id);
                return Ok(response::Redirect::to(&uri).into_response());
            }
            Ok(None) => {
                log::info!("tagged quote selection was empty");
            }
            Err(e) => {
                log::error!("tagged quote selection database error: {}", e);
                panic!("tagged quote selection database error");
            }
        }
    }

    let quote_result = quote::get_random(&db).await;
    match quote_result {
        Ok(id) => {
            let uri = format!("/?id={}", id);
            Ok(response::Redirect::to(&uri).into_response())
        }
        Err(e) => {
            log::error!("quote selection failed: {}", e);
            panic!("quote selection failed");
        }
    }
}