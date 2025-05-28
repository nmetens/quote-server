extern crate serde_json;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum QuoteError {
    #[error("could not find quote file: {0}")]
    QuotesNotFound(#[from] std::io::Error),
    #[error("could not read quote file: {0}")]
    QuoteMisformat(#[from] serde_json::Error),
    #[error("invalid database uri: {0}")]
    InvalidDbUri(String),
}
