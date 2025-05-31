CREATE TABLE tags (
    quote_id INTEGER NOT NULL,
    tag TEXT NOT NULL,
    FOREIGN KEY (quote_id) REFERENCES quotes(id) ON DELETE CASCADE
);
