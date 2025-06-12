use crate::*;

use askama::Template;

#[derive(Template)]
#[template(path = "index.html")]
pub struct IndexTemplate {
    quote: Quote,
    tags: String,
}

impl IndexTemplate {
    pub fn new(quote: Quote, tags: String) -> Self {
        Self {
            quote,
            tags,
        }
    }
}
