use crate::*;
use askama::Template;

#[derive(Template)]
#[template(path = "index.html")]
pub struct IndexTemplate<'a> {
    quote: &'a Quote,
    stylesheet: &'static str,
    tags: String,
}

impl<'a> IndexTemplate<'a> {
    pub fn quote(quote: &'a Quote, tags: String) -> Self {
        Self {
            quote,
            stylesheet: "/quote.css",
            tags,
        }
    }
}
