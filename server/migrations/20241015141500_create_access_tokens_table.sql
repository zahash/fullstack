CREATE TABLE access_tokens (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    access_token_hash BLOB NOT NULL UNIQUE,
    user_id INTEGER NOT NULL,
    created_at DATETIME NOT NULL,
    expires_at DATETIME,
    FOREIGN KEY (user_id) REFERENCES users (id)
);
