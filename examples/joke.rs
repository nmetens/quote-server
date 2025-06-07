// This code originally borrowed from the leptos crate
// examples, where variants appear throughout.

use leptos::prelude::*;
use serde::{Serialize, Deserialize};
use std::collections::HashSet;

#[derive(Serialize, Deserialize)]
pub struct Joke {
    pub id: String,
    pub whos_there: String,
    pub answer_who: String,
    pub tags: HashSet<String>,
    pub source: String,
}



pub async fn fetch(endpoint: String) -> Result<Joke, Error> {
    use reqwasm::http::Request;

    let ep = format!(
        "http://localhost:3000/api/v1/{}",
        endpoint,
    );
    let result = Request::get(&ep)
        .send()
        .await?
        // convert it to JSON
        .json()
        .await?;
    Ok(result)
}
