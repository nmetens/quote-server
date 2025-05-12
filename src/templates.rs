use crate::*;
use askama::Template;

#[derive(Template)]
#[template(path = "index.html")]
pub struct IndexTemplate<'a> {
    quote: &'a Quote,
    //stylesheet: &'static str,
}

impl<'a> IndexTemplate<'a> {
    pub fn quote(quote: &'a Quote) -> Self {
        Self {
            quote,
            //stylesheet: "/quote.css",
        }
    }
}
