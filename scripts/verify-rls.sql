-- QuillSpace RLS Verification Script
-- Comprehensive check of Row-Level Security configuration

\echo '🔒 QuillSpace RLS Verification'
\echo '=============================='

-- 1. Check all tables and their RLS status
\echo ''
\echo '📊 Table RLS Status:'
SELECT 
    tablename,
    rowsecurity as rls_enabled,
    (SELECT COUNT(*) FROM pg_policies p WHERE p.tablename = t.tablename) as policies
FROM pg_tables t 
WHERE schemaname = 'public' 
ORDER BY tablename;

-- 2. Check all RLS policies
\echo ''
\echo '🛡️  RLS Policies:'
SELECT 
    schemaname,
    tablename,
    policyname,
    cmd as operation,
    qual as condition
FROM pg_policies 
ORDER BY tablename, policyname;

-- 3. Check current user privileges
\echo ''
\echo '👤 Current User Info:'
SELECT 
    current_user as username,
    rolsuper as is_superuser,
    rolbypassrls as bypasses_rls
FROM pg_roles 
WHERE rolname = current_user;

-- 4. Test data counts
\echo ''
\echo '📈 Data Summary:'
SELECT 'tenants' as table_name, COUNT(*) as records FROM tenants
UNION ALL
SELECT 'users' as table_name, COUNT(*) as records FROM users
UNION ALL
SELECT 'content' as table_name, COUNT(*) as records FROM content
UNION ALL
SELECT 'sites' as table_name, COUNT(*) as records FROM sites
UNION ALL
SELECT 'pages' as table_name, COUNT(*) as records FROM pages
UNION ALL
SELECT 'assets' as table_name, COUNT(*) as records FROM assets
ORDER BY table_name;

-- 5. Test RLS context functions
\echo ''
\echo '🔧 RLS Context Functions:'
SELECT 
    current_setting('rls.tenant_id', true) as current_tenant,
    current_setting('rls.user_id', true) as current_user_id;

-- 6. Test tenant isolation with Yasin's context
\echo ''
\echo '🧪 Testing Yasin Tenant Context:'
SELECT set_config('rls.tenant_id', '11111111-1111-1111-1111-111111111111', true);
SELECT 'content_visible_to_yasin' as test, COUNT(*) as count FROM content;
SELECT 'users_visible_to_yasin' as test, COUNT(*) as count FROM users;

-- 7. Test tenant isolation with Joey's context
\echo ''
\echo '🧪 Testing Joey Tenant Context:'
SELECT set_config('rls.tenant_id', '22222222-2222-2222-2222-222222222222', true);
SELECT 'content_visible_to_joey' as test, COUNT(*) as count FROM content;
SELECT 'users_visible_to_joey' as test, COUNT(*) as count FROM users;

-- 8. Test no context (should see nothing or default)
\echo ''
\echo '🧪 Testing No Tenant Context:'
SELECT set_config('rls.tenant_id', '', true);
SELECT 'content_no_context' as test, COUNT(*) as count FROM content;
SELECT 'users_no_context' as test, COUNT(*) as count FROM users;

-- 9. Check specific tenant data
\echo ''
\echo '📋 Tenant Data Breakdown:'
SELECT 
    t.name as tenant_name,
    t.slug as tenant_slug,
    COUNT(DISTINCT u.id) as user_count,
    COUNT(DISTINCT c.id) as content_count
FROM tenants t
LEFT JOIN users u ON t.id = u.tenant_id
LEFT JOIN content c ON t.id = c.tenant_id
GROUP BY t.id, t.name, t.slug
ORDER BY t.name;

\echo ''
\echo '✅ RLS Verification Complete!'
\echo 'Review the results above to ensure proper tenant isolation.'
