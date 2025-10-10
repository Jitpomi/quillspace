-- QuillSpace Complete Setup Verification
-- This script verifies that the complete RLS system is working as documented

\echo '🔒 QuillSpace Complete Setup Verification'
\echo '========================================='
\echo ''

-- 1. Check RLS functions exist
\echo '1️⃣ Checking RLS Functions:'
SELECT 
    proname as function_name,
    CASE WHEN proname IS NOT NULL THEN '✅ EXISTS' ELSE '❌ MISSING' END as status
FROM (VALUES 
    ('current_tenant_id'),
    ('current_user_id'), 
    ('get_tenant_isolation_mode'),
    ('user_can_access_resource'),
    ('set_tenant_user_isolation'),
    ('test_tenant_isolation')
) AS expected_functions(fname)
LEFT JOIN pg_proc ON proname = fname
ORDER BY fname;

\echo ''

-- 2. Check RLS policies
\echo '2️⃣ Checking RLS Policies:'
SELECT 
    tablename,
    COUNT(*) as policy_count,
    CASE 
        WHEN COUNT(*) > 0 THEN '✅ PROTECTED'
        ELSE '❌ NO POLICIES'
    END as status
FROM pg_policies 
WHERE schemaname = 'public'
AND tablename IN ('tenants', 'users', 'content', 'sites', 'pages', 'templates', 'assets')
GROUP BY tablename
ORDER BY tablename;

\echo ''

-- 3. Check tenant isolation modes
\echo '3️⃣ Checking Tenant Isolation Settings:'
SELECT * FROM tenant_isolation_settings;

\echo ''

-- 4. Test tenant isolation
\echo '4️⃣ Testing Tenant Isolation:'
SELECT * FROM test_tenant_isolation();

\echo ''

-- 5. Check RLS status
\echo '5️⃣ RLS Status Monitor:'
SELECT * FROM rls_status_monitor;

\echo ''

-- 6. Verify context functions work
\echo '6️⃣ Testing Context Functions:'
SELECT 
    'No Context' as test_case,
    current_tenant_id() as tenant_id,
    current_user_id() as user_id,
    get_tenant_isolation_mode() as isolation_mode;

-- Set Yasin's context and test
SELECT set_config('app.current_tenant_id', '11111111-1111-1111-1111-111111111111', true);
SELECT set_config('rls.user_id', 'bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb', true);

SELECT 
    'Yasin Context' as test_case,
    current_tenant_id() as tenant_id,
    current_user_id() as user_id,
    get_tenant_isolation_mode() as isolation_mode;

\echo ''
\echo '✅ Complete Setup Verification Finished!'
\echo 'Review results above to confirm full RLS compliance.'
