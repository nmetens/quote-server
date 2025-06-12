// This code originally borrowed from the leptos crate
// examples, where variants appear throughout.

use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

// The famous quote struct. Contains and id, a quote, and its author:
#[derive(Serialize, Deserialize)]
pub struct Quote {
    pub id: String,
    pub quote: String,
    pub author: String,
    tags: HashSet<String>,
}

pub async fn fetch(endpoint: String) -> Result<Quote, Error> {
    use reqwasm::http::Request;

    let ep = format!("http://localhost:8000/api/v1/{}", endpoint,);
    println!("endpoint: {}", endpoint);
    let result = Request::get(&ep)
        .send()
        .await?
        // convert it to JSON
        .json()
        .await?;
    Ok(result)
}
