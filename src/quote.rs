use std::collections::HashSet;
use std::ops::Deref;
use std::path::Path;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct JsonQuote {
    id: i32,
    quote: String,
    author: String,
    tags: Option<Vec<HashSet<String>>>,
}

pub struct Quote {
    pub id: i32,
    pub quote: String,
    pub author: String,
}

pub fn read_quotes<P: AsRef<Path>>(quotes_path: P) -> 
    Result<Vec<JsonQuote>, Box<dyn std::error::Error>> {

    let f = std::fs::File::open(quotes_path.as_ref())?;
    let quotes = serde_json::from_reader(f)?;
    Ok(quotes)
}

impl JsonQuote {
    pub fn to_quote(&self) -> Quote {
        let quote = Quote {
            id: self.id.clone(),
            quote: self.quote.clone(),
            author: self.author.clone(),
        };
        quote
    }
}
