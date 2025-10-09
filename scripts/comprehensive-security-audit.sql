-- QuillSpace Comprehensive Security Audit
-- This script systematically verifies every security aspect

\echo '🔒 COMPREHENSIVE SECURITY AUDIT'
\echo '================================'
\echo ''

-- STEP 1: Table Ownership Audit
\echo '1️⃣ TABLE OWNERSHIP AUDIT'
\echo '========================'
SELECT 
    tablename,
    tableowner,
    CASE 
        WHEN tableowner = 'quillspace' THEN '❌ VULNERABLE (quillspace owns table)'
        WHEN tableowner = 'postgres' THEN '✅ SECURE (postgres owns table)'
        ELSE '⚠️ UNKNOWN OWNER'
    END as security_status
FROM pg_tables 
WHERE schemaname = 'public' 
ORDER BY tablename;

\echo ''

-- STEP 2: RLS Status Audit
\echo '2️⃣ ROW LEVEL SECURITY STATUS'
\echo '============================='
SELECT 
    tablename,
    rowsecurity as rls_enabled,
    (SELECT COUNT(*) FROM pg_policies p WHERE p.tablename = t.tablename) as policy_count,
    CASE 
        WHEN rowsecurity = true AND (SELECT COUNT(*) FROM pg_policies p WHERE p.tablename = t.tablename) > 0 
        THEN '✅ PROTECTED'
        WHEN rowsecurity = true AND (SELECT COUNT(*) FROM pg_policies p WHERE p.tablename = t.tablename) = 0
        THEN '⚠️ RLS ON BUT NO POLICIES'
        WHEN rowsecurity = false
        THEN '❌ RLS DISABLED'
        ELSE '❓ UNKNOWN STATUS'
    END as security_status
FROM pg_tables t
WHERE schemaname = 'public' 
AND tablename IN ('tenants', 'users', 'content', 'sites', 'pages', 'templates', 'assets', 'domains', 'widgets')
ORDER BY tablename;

\echo ''

-- STEP 3: User Privileges Audit
\echo '3️⃣ USER PRIVILEGES AUDIT'
\echo '========================'
SELECT 
    rolname,
    rolsuper as is_superuser,
    rolbypassrls as bypasses_rls,
    rolcreaterole as can_create_roles,
    rolcreatedb as can_create_db,
    CASE 
        WHEN rolsuper = true OR rolbypassrls = true THEN '❌ DANGEROUS PRIVILEGES'
        ELSE '✅ SAFE PRIVILEGES'
    END as security_status
FROM pg_roles 
WHERE rolname IN ('quillspace', 'postgres', 'quillspace_app', 'quillspace_readonly')
ORDER BY rolname;

\echo ''

-- STEP 4: RLS Policy Details
\echo '4️⃣ RLS POLICY DETAILS'
\echo '====================='
SELECT 
    schemaname,
    tablename,
    policyname,
    cmd as operation,
    qual as condition
FROM pg_policies 
WHERE tablename IN ('users', 'content', 'sites', 'pages', 'templates', 'assets')
ORDER BY tablename, policyname;

\echo ''

-- STEP 5: Data Distribution Check
\echo '5️⃣ DATA DISTRIBUTION CHECK'
\echo '============================'
\echo 'Checking if data is properly distributed by tenant...'

SELECT 'TENANTS:' as table_name, COUNT(*) as total_records FROM tenants
UNION ALL
SELECT 'USERS by tenant:', NULL FROM (SELECT 1) as dummy WHERE false
UNION ALL
SELECT '  - ' || COALESCE(t.name, 'Unknown') as tenant_name, COUNT(u.id) as user_count
FROM tenants t
LEFT JOIN users u ON t.id = u.tenant_id
GROUP BY t.id, t.name
UNION ALL
SELECT 'CONTENT by tenant:', NULL FROM (SELECT 1) as dummy WHERE false
UNION ALL
SELECT '  - ' || COALESCE(t.name, 'Unknown') as tenant_name, COUNT(c.id) as content_count
FROM tenants t
LEFT JOIN content c ON t.id = c.tenant_id
GROUP BY t.id, t.name;

\echo ''
\echo '✅ SECURITY AUDIT COMPLETE'
\echo '=========================='
\echo 'Review all sections above for security vulnerabilities.'
\echo 'Any items marked with ❌ require immediate attention.'
