INSERT INTO permissions (permission, description) VALUES
('post:/access-token/generate',         'Generate a new Access Token'),
('get:/permissions',                    'Get a list of permissions held by the Principal'),
('post:/permissions/assign',            'Assign a permission to an Assignee'),
('post:/rotate-key',                    'Rotate the Secret key'),
('get:/sysinfo',                        'Get system information')
ON CONFLICT (permission) DO NOTHING;


INSERT INTO permission_groups ([group], description) VALUES
('root',        'for superuser access'),
('admin',       'for site administrators'),
('signup',      'for users that just signed up')
ON CONFLICT ([group]) DO NOTHING;


WITH mapping([group], permission) AS (
  VALUES
    ('signup',    'post:/access-token/generate'),
    ('signup',    'get:/permissions'),
    ('signup',    'post:/permissions/assign'),

    ('admin',     'post:/access-token/generate'),
    ('admin',     'get:/permissions'),
    ('admin',     'post:/permissions/assign'),
    ('admin',     'post:/rotate-key'),
    ('admin',     'get:/sysinfo')
)
INSERT INTO permission_group_association (permission_id, permission_group_id)
SELECT p.id, pg.id
FROM mapping m
INNER JOIN permissions p ON p.permission = m.permission
INNER JOIN permission_groups pg ON pg.[group] = m.[group]
ON CONFLICT (permission_id, permission_group_id) DO NOTHING;


-- Assign ALL permissions to the 'root' group
INSERT INTO permission_group_association (permission_id, permission_group_id)
SELECT p.id, pg.id
FROM permissions p
CROSS JOIN permission_groups pg
WHERE pg.[group] = 'root'
ON CONFLICT (permission_id, permission_group_id) DO NOTHING;
