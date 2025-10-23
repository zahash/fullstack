CREATE TABLE sessions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    session_id_hash BLOB NOT NULL UNIQUE,
    user_id INTEGER NOT NULL,
    created_at DATETIME NOT NULL,
    expires_at DATETIME NOT NULL,
    user_agent TEXT,
    FOREIGN KEY (user_id) REFERENCES users (id),
    CHECK (expires_at >= created_at)
);
