CREATE TABLE access_tokens (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    access_token_hash BLOB NOT NULL UNIQUE,
    user_id INTEGER NOT NULL,
    created_at DATETIME NOT NULL,
    expires_at DATETIME NOT NULL,
    FOREIGN KEY (user_id) REFERENCES users (id),
    UNIQUE (user_id, name),
    CHECK (expires_at >= created_at)
);
CREATE INDEX idx__access_tokens__user_id__name ON access_tokens (user_id, name);
