CREATE TABLE permissions(
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    permission TEXT NOT NULL UNIQUE,
    description TEXT
);

CREATE TABLE permission_groups(
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    [group] TEXT NOT NULL UNIQUE,
    description TEXT
);

CREATE TABLE permission_group_association(
    permission_id INTEGER NOT NULL,
    permission_group_id INTEGER NOT NULL,
    PRIMARY KEY (permission_id, permission_group_id),
    FOREIGN KEY (permission_id) REFERENCES permissions (id) ON DELETE CASCADE,
    FOREIGN KEY (permission_group_id) REFERENCES permission_groups (id) ON DELETE CASCADE
);
CREATE INDEX idx__permission_group_association__permission_id ON permission_group_association (permission_id);
CREATE INDEX idx__permission_group_association__permission_group_id ON permission_group_association (permission_group_id);

CREATE TABLE user_permissions(
    user_id INTEGER NOT NULL,
    permission_id INTEGER NOT NULL,
    PRIMARY KEY (user_id, permission_id),
    FOREIGN KEY (user_id) REFERENCES users (id) ON DELETE CASCADE,
    FOREIGN KEY (permission_id) REFERENCES permissions (id) ON DELETE CASCADE
);
CREATE INDEX idx__user_permissions__user_id ON user_permissions (user_id);
CREATE INDEX idx__user_permissions__permission_id ON user_permissions (permission_id);

CREATE TABLE access_token_permissions(
    access_token_id INTEGER NOT NULL,
    permission_id INTEGER NOT NULL,
    PRIMARY KEY (access_token_id, permission_id),
    FOREIGN KEY (access_token_id) REFERENCES access_tokens (id) ON DELETE CASCADE,
    FOREIGN KEY (permission_id) REFERENCES permissions (id) ON DELETE CASCADE
);
CREATE INDEX idx__access_token_permissions__access_token_id ON access_token_permissions (access_token_id);
CREATE INDEX idx__access_token_permissions__permission_id ON access_token_permissions (permission_id);

CREATE TABLE permissions_audit_log(
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    assigner_type TEXT NOT NULL,
    assigner_id INTEGER NOT NULL,
    assignee_type TEXT NOT NULL,
    assignee_id INTEGER NOT NULL,
    permission_id INTEGER NOT NULL,
    action TEXT NOT NULL,
    datetime DATETIME NOT NULL,
    FOREIGN KEY (permission_id) REFERENCES permissions (id) ON DELETE CASCADE,
    CHECK (assigner_type IN ('user', 'access_token')),
    CHECK (assignee_type IN ('user', 'access_token')),
    CHECK (action IN ('assign', 'revoke'))
);
CREATE INDEX idx__permissions_audit_log__datetime ON permissions_audit_log (datetime);
CREATE INDEX idx__permissions_audit_log__assigner ON permissions_audit_log (assigner_type, assigner_id, datetime);
CREATE INDEX idx__permissions_audit_log__assignee ON permissions_audit_log (assignee_type, assignee_id, datetime);
CREATE INDEX idx__permissions_audit_log__permission ON permissions_audit_log (permission_id, datetime);
