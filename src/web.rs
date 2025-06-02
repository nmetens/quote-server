/// This file handles the url parameters accordingly. If the ur; has an id, we
/// check that the quote with that id exists in the database and return it displayed
/// one the page. We could have a tag (theme) instead, and then display a random 
/// quote with that theme, or we could have the /random-quote url paramter in which
/// we simply query the query table in the database for a random quote and redirect
/// the user to see that quote.

use crate::*;

// Struct defining the url parameters. Both the id and tags are options since they can
// be either an id, a tag, or none.
// The struct will allow us to parse the query parameters.
#[derive(Deserialize)]
pub struct GetQuoteParams {
    id: Option<String>,
    tags: Option<String>,
}

// Function that has params app_state which locks the AppState for safe handling
// of async state which allows safe reading and writing between resources. The params
// allows us to parse the url params using the GetQuoteParams struct.
pub async fn get_quote(
    State(app_state): State<Arc<RwLock<AppState>>>,
    Query(params): Query<GetQuoteParams>,
) -> Result<response::Response, http::StatusCode> {
    // Lock the app for writing and clone the database pool for queries:
    let mut app_writer = app_state.write().await;
    let db = app_writer.db.clone();

    // Specified. If the some if is specified in the parameters list, then 
    // get the quote from the database:
    if let GetQuoteParams { id: Some(id), .. } = params {

        // Fetch the quote with the id, if it exists:
        let quote_result = quote::get(&db, &id).await;

        // Match the resulting quote:
        let result = match quote_result {
            // Return the quote object and tags separated by commas:
            Ok((quote, tags)) => {
                let tag_string = tags.join(", ");

                app_writer.current_quote = quote.clone(); // Write the quote into the app.

                // Put the quote into the index.html page.
                let quote = IndexTemplate::new(quote.clone(), tag_string);

                Ok(response::Html(quote.to_string()).into_response()) // Return the html response with the quote and tags.
            }

            // There was no quote with the given id parameter: return 404 not found onto the index.html.
            Err(e) => {
                log::warn!("quote fetch failed: {}", e);
                Err(http::StatusCode::NOT_FOUND)
            }
        };
        return result; // Return the result if it wasn;t an error.
    }

    
    // If the parameters has no id, but a tag a.k.a a theme instead, come here:
    if let GetQuoteParams { tags: Some(tags), .. } = params {
        log::info!("quote theme: {}", tags); // Display the theme to the console.

        // Create a new string object that is mutable.
        let mut tags_string = String::new();

        // For each character in the tags provided as a parameter, check if the characters are 
        // valid and in the alphabet, also if there is more than one theme provided. No special characters.
        for c in tags.chars() {
            if c.is_alphabetic() || c == ',' {
                let cl: String = c.to_lowercase().collect(); // Put all characters to lovercase.
                tags_string.push_str(&cl); // Push all lowercase chars into the string object.
            }
        }

        // Call the get_tagged quote method with the database pool containing the quotes table, and the 
        // tags string which could have multiple themes separated by commas:
        let quote_result = quote::get_tagged(&db, tags_string.split(',')).await;

        // Match the quote result to either an id, or none and return accordingly:
        match quote_result {
            Ok(Some(id)) => { // The resulting quote from the databse with the tag exists and returned a result.
                let uri = format!("/?id={}", id); // Set the uri to show the quote id of the quote_result.
                return Ok(response::Redirect::to(&uri).into_response()); // Redirect the page to the uri created.
            }

            Ok(None) => { // No id found from the tag provided. Log to the console.
                log::info!("tagged quote selection was empty");
            }

            Err(e) => { // Database error.
                log::error!("tagged quote selection database error: {}", e);
                panic!("tagged quote selection database error");
            }
        }
    }

    // Get a random quote from the database:
    let quote_result = quote::get_random(&db).await;

    // Match the random quote result to one of the following:
    match quote_result {
        Ok(id) => { // An id was correctly returned and so display that id on the page uri.
            let uri = format!("/?id={}", id);
            Ok(response::Redirect::to(&uri).into_response())
        }

        Err(e) => { // An error occured in the database:
            log::error!("quote selection failed: {}", e);
            panic!("quote selection failed");
        }
    }
}