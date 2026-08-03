-- Raksha: assign target roles & membership to per-role seed accounts
UPDATE users SET role='super_admin' WHERE email='superadmin@raksha.local';
UPDATE users SET role='admin'       WHERE email='tenantadmin@raksha.local';
UPDATE users SET role='analyst'     WHERE email='analyst@raksha.local';
UPDATE users SET role='operator'    WHERE email='operator@raksha.local';
UPDATE users SET role='viewer'      WHERE email='viewer@raksha.local';
UPDATE users SET role='viewer'      WHERE email='ops@raksha.local';

-- Fix user_roles membership to the matching named role
UPDATE user_roles ur SET role_id = r.id
FROM users u, roles r, tenants t
WHERE ur.user_id = u.id
  AND t.id = '00000000-0000-0000-0000-000000000001'
  AND ur.org_id = t.id
  AND CASE u.email
        WHEN 'superadmin@raksha.local' THEN r.name='super_admin'
        WHEN 'tenantadmin@raksha.local' THEN r.name='tenant_admin'
        WHEN 'analyst@raksha.local' THEN r.name='analyst'
        WHEN 'operator@raksha.local' THEN r.name='operator'
        WHEN 'viewer@raksha.local' THEN r.name='viewer'
        WHEN 'ops@raksha.local' THEN r.name='viewer'
        ELSE false END;

-- Show result
SELECT u.email, u.name, u.role AS users_role, r.name AS membership
FROM users u
JOIN user_roles ur ON ur.user_id = u.id
JOIN roles r ON r.id = ur.role_id
ORDER BY u.email;
