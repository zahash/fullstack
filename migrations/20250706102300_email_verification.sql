ALTER TABLE users
ADD COLUMN email_verified BOOLEAN NOT NULL DEFAULT 0;

CREATE TABLE email_verification_tokens (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    email TEXT NOT NULL UNIQUE,
    token_hash BLOB NOT NULL UNIQUE,
    created_at DATETIME NOT NULL,
    expires_at DATETIME NOT NULL,
    FOREIGN KEY (email) REFERENCES users (email) ON DELETE CASCADE
);
