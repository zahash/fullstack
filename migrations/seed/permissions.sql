INSERT INTO permissions (permission, description) VALUES
('post:/access-token/generate',         'Generate a new Access Token'),
('get:/permissions',                    'Get a list of permissions held by the Principal'),
('post:/permissions/assign',            'Assign a permission to an Assignee'),
('get:/sysinfo',                        'Get system information')
ON CONFLICT (permission) DO NOTHING;


INSERT INTO permission_groups (group, description) VALUES
('signup',      'for users that just signed up'),
('admin',       'for site administrators')
ON CONFLICT (group) DO NOTHING;


WITH mapping(permission, [group]) AS (
  VALUES
    ('post:/access-token/generate',         'signup'),
    ('get:/permissions',                    'admin'),
    ('post:/permissions/assign',            'admin'),
    ('get:/sysinfo',                        'admin')
)
INSERT INTO permission_group_association (permission_id, permission_group_id)
SELECT p.id, pg.id
FROM mapping m
INNER JOIN permissions p ON p.permission = m.permission
INNER JOIN permission_groups pg ON pg.[group] = m.[group]
ON CONFLICT (permission_id, permission_group_id) DO NOTHING;
