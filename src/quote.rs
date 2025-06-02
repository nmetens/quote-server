/// This file defines the JsonQuote and the Quote structs. The Json Quote allows movement of quote
/// data over the api. The Quote struct is used to create quotes and grab them from the database.
/// This file has the functions that get the tags and Quotes from the database.

use crate::*;

use std::collections::HashSet;
use std::ops::Deref;
use std::path::Path;

use crate::QuoteError;

use serde::Deserialize;

// Struct that sends Json quotes over the api:
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct JsonQuote {
    id: String, // Unique id "1", "2", etc.
    quote: String, // The famous quote.
    author: String, // Author of the quote.
    tags: HashSet<String>, // Set of tags (themes) of the quote: ['love', 'life']
}

// The famous quote struct. Contains and id, a quote, and its author:
#[derive(Clone)]
pub struct Quote {
    pub id: String,
    pub quote: String,
    pub author: String,
}

// Read quotes from the quotes.json and parse them into JsonQuote objects:
pub fn read_quotes<P: AsRef<Path>>(quotes_path: P) -> Result<Vec<JsonQuote>, FamousQuoteError> {
    let f = std::fs::File::open(quotes_path.as_ref())?;
    let quotes = serde_json::from_reader(f)?;
    Ok(quotes)
}

// Implementation methods for the json quote struct:
impl JsonQuote {
    // The constructor that tags the tags and the Quote to create a JsonQuote object:
    pub fn new(quote: Quote, tags: Vec<String>) -> Self {
        let tags = tags.into_iter().collect(); // The collection of tags... ['love', 'life', 'marriage']
        Self {
            id: quote.id,
            quote: quote.quote,
            author: quote.author,
            tags,
        }
    }

    // Creates a Quote object from the JsonQuote instance and returns it plus the tags.
    pub fn to_quote(&self) -> (Quote, impl Iterator<Item = &str>) {
        let quote = Quote {
            id: self.id.clone(),
            quote: self.quote.clone(),
            author: self.author.clone(),
        };
        let tags = self.tags.iter().map(String::deref);
        (quote, tags) // Returns the tuple of the Quote object and the tags
    }
}

// Converts a JsonQuote object into an http response type:
impl axum::response::IntoResponse for &JsonQuote {
    fn into_response(self) -> axum::response::Response {
        (StatusCode::OK, axum::Json(&self)).into_response()
    }
}

// Given the database pool and quote id, get the quote with that id by querying the database
// and returning the quote object and the tags in tuple form:
pub async fn get(db: &SqlitePool, quote_id: &str) -> Result<(Quote, Vec<String>), sqlx::Error> {
    // Get the quote from the quotes table by the id:
    let quote = sqlx::query_as("select * from quotes where id = $1;")
        .bind(quote_id)
        .fetch_one(db)
        .await?;

    // Get the tag from the tags table where the quote id is matched:
    let tags: Vec<String> = sqlx::query_scalar!("select tag from tags where quote_id = $1;", quote_id)
        .fetch_all(db)
        .await?;

    Ok((quote, tags)) // Return the tuple.
}

// Given the database pool and the tags, get a quote from the db that matches that tag:
pub async fn get_tagged<'a, I>(db: &SqlitePool, tags: I) -> Result<Option<String>, sqlx::Error>
    where I: Iterator<Item=&'a str>
{
    let mut qtx = db.begin().await?; // Begin a transaction with the databse.

    // Drop the qtags table (if it exists), then create a temporary table called qtags with one column: tag.
    sqlx::query("drop table if exists qtags;").execute(&mut *qtx).await?;
    sqlx::query("create temporary table qtags (tag varchar(200));")
        .execute(&mut *qtx)
        .await?;

    // For each tag in the tags list (given as a paramter to this method), insert that tag into the temporary table:
    for tag in tags {
        sqlx::query("insert into qtags values ($1);")
            .bind(tag)
            .execute(&mut *qtx)
            .await?;
    }

    // Join the temporary qtags table with the persistant tags table on the tag column.
    // Select only the unique tags from that join, and select the top one at random:
    let quote_ids = sqlx::query("
            select distinct quote_id 
            from tags 
            join qtags 
            on tags.tag = qtags.tag 
            order by random() 
            limit 1;")
        .fetch_all(&mut *qtx)
        .await?;

    // Get the quote ids generated from the previous query:
    let nquote_ids = quote_ids.len();

    // If there is one quote id, return it as the result.
    let result = 
        if nquote_ids == 1 {
            Some(quote_ids[0].get(0))
        } else {
            None
        };

    qtx.commit().await?; // End and commit the transaction that started at the top of this function.

    // Return the quote_id that had a matching theme (tag).
    Ok(result)
}

// Query the database and get a random quote id:
pub async fn get_random(db: &SqlitePool) -> Result<String, sqlx::Error> {
    sqlx::query_scalar!("select id from quotes order by random() limit 1;")
        .fetch_one(db)
        .await
}