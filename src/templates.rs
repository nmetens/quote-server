/// This file is the askama template renderer that uses the index.html file,
/// and the quote.css stylesheet. This file has the struct definition and the
/// constructor that captures the data that the front end will have access to
/// to display the quote.

use crate::*;
use askama::Template;

// Create the IndexTemplate struct that has the path index.html.
#[derive(Template)]
#[template(path = "index.html")] // Use the html found in the templates folder in the static dir.
pub struct IndexTemplate {
    quote: Quote, // The quote object that will be used to display the quote, author, and theme in the index.html.
    stylesheet: &'static str, // Reference to the quote.css styling file.
    tags: String, // The theme of the quote to display.
}

// The constructor that initializes the struct with the 
// Quote object, and the tags.
impl IndexTemplate {
    pub fn new(quote: Quote, tags: String) -> Self {
        Self {
            quote, // The quote object from the database.
            stylesheet: "/quote.css", // The assets/static/quote.css file.
            tags, // Theme of the quote.
        }
    }
}