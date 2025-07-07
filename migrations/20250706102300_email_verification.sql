ALTER TABLE users
ADD COLUMN email_verified BOOLEAN NOT NULL DEFAULT 0;

CREATE TABLE email_verification_tokens (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    token_hash BLOB NOT NULL UNIQUE,
    user_id INTEGER NOT NULL,
    created_at DATETIME NOT NULL,
    expires_at DATETIME NOT NULL,
    FOREIGN KEY (user_id) REFERENCES users (id)
);
