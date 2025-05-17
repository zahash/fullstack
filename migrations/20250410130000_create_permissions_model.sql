CREATE TABLE permissions(
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    permission TEXT NOT NULL UNIQUE,
    description TEXT
);

CREATE TABLE user_permissions(
    user_id INTEGER NOT NULL,
    permission_id INTEGER NOT NULL,
    PRIMARY KEY (user_id, permission_id),
    FOREIGN KEY (user_id) REFERENCES users (id) ON DELETE CASCADE,
    FOREIGN KEY (permission_id) REFERENCES permissions (id) ON DELETE CASCADE
);

CREATE TABLE access_token_permissions(
    access_token_id INTEGER NOT NULL,
    permission_id INTEGER NOT NULL,
    PRIMARY KEY (access_token_id, permission_id),
    FOREIGN KEY (access_token_id) REFERENCES access_tokens (id) ON DELETE CASCADE,
    FOREIGN KEY (permission_id) REFERENCES permissions (id) ON DELETE CASCADE
);

CREATE INDEX idx__user_permissions__user_id ON user_permissions (user_id);
CREATE INDEX idx__user_permissions__permission_id ON user_permissions (permission_id);

CREATE INDEX idx__access_token_permissions__access_token_id ON access_token_permissions (access_token_id);
CREATE INDEX idx__access_token_permissions__permission_id ON access_token_permissions (permission_id);
