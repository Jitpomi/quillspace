-- QuillSpace RLS Testing Script
-- Comprehensive test suite for Row-Level Security policies
-- Tests tenant isolation across all multi-tenant tables

\echo '🔒 QuillSpace RLS Testing Suite'
\echo '================================'
\echo ''

-- Test setup: Create test data for RLS validation
\echo '📋 Setting up test data...'

-- Insert test sites for both tenants
INSERT INTO sites (id, tenant_id, name, subdomain, description) VALUES
    ('test0001-site-yasin-0000-000000000001', '11111111-1111-1111-1111-111111111111', 'Yasin Test Site', 'yasin-test', 'Test site for Yasin'),
    ('test0002-site-joey-0000-000000000002', '22222222-2222-2222-2222-222222222222', 'Joey Test Site', 'joey-test', 'Test site for Joey')
ON CONFLICT (id) DO UPDATE SET 
    name = EXCLUDED.name,
    description = EXCLUDED.description;

-- Insert test pages for both sites
INSERT INTO pages (id, site_id, slug, title, puck_data) VALUES
    ('page0001-yasin-test-0000-000000000001', 'test0001-site-yasin-0000-000000000001', 'home', 'Yasin Home Page', '{"content": []}'),
    ('page0002-yasin-test-0000-000000000002', 'test0001-site-yasin-0000-000000000001', 'about', 'Yasin About Page', '{"content": []}'),
    ('page0003-joey-test-0000-000000000003', 'test0002-site-joey-0000-000000000002', 'home', 'Joey Home Page', '{"content": []}'),
    ('page0004-joey-test-0000-000000000004', 'test0002-site-joey-0000-000000000002', 'about', 'Joey About Page', '{"content": []}')
ON CONFLICT (site_id, slug) DO UPDATE SET 
    title = EXCLUDED.title,
    puck_data = EXCLUDED.puck_data;

-- Insert test assets for both tenants
INSERT INTO assets (id, tenant_id, site_id, filename, original_filename, mime_type, file_size, storage_path) VALUES
    ('asset001-yasin-test-0000-000000000001', '11111111-1111-1111-1111-111111111111', 'test0001-site-yasin-0000-000000000001', 'yasin-logo.png', 'logo.png', 'image/png', 12345, '/uploads/yasin/logo.png'),
    ('asset002-joey-test-0000-000000000002', '22222222-2222-2222-2222-222222222222', 'test0002-site-joey-0000-000000000002', 'joey-banner.jpg', 'banner.jpg', 'image/jpeg', 67890, '/uploads/joey/banner.jpg')
ON CONFLICT (id) DO UPDATE SET 
    filename = EXCLUDED.filename,
    storage_path = EXCLUDED.storage_path;

\echo '✅ Test data created'
\echo ''

-- Function to test RLS isolation
CREATE OR REPLACE FUNCTION test_rls_isolation()
RETURNS TABLE(
    test_name TEXT,
    tenant_context TEXT,
    table_name TEXT,
    expected_count INTEGER,
    actual_count INTEGER,
    status TEXT
) AS $$
DECLARE
    yasin_tenant_id UUID := '11111111-1111-1111-1111-111111111111';
    joey_tenant_id UUID := '22222222-2222-2222-2222-222222222222';
    test_result RECORD;
BEGIN
    \echo '🧪 Testing RLS Policies...'
    \echo ''
    
    -- Test 1: Yasin's context - should only see Yasin's data
    \echo '👤 Testing Yasin''s tenant context...'
    
    -- Set Yasin's tenant context
    PERFORM set_config('rls.tenant_id', yasin_tenant_id::text, true);
    
    -- Test sites table
    SELECT COUNT(*)::INTEGER INTO test_result FROM sites;
    RETURN QUERY SELECT 
        'Yasin Context'::TEXT,
        'Yasin Tenant'::TEXT,
        'sites'::TEXT,
        1::INTEGER, -- Expected: 1 site (Yasin's)
        test_result.count::INTEGER,
        CASE WHEN test_result.count = 1 THEN '✅ PASS' ELSE '❌ FAIL' END::TEXT;
    
    -- Test pages table (should see pages from Yasin's sites only)
    SELECT COUNT(*)::INTEGER INTO test_result FROM pages;
    RETURN QUERY SELECT 
        'Yasin Context'::TEXT,
        'Yasin Tenant'::TEXT,
        'pages'::TEXT,
        2::INTEGER, -- Expected: 2 pages (Yasin's home + about)
        test_result.count::INTEGER,
        CASE WHEN test_result.count = 2 THEN '✅ PASS' ELSE '❌ FAIL' END::TEXT;
    
    -- Test assets table
    SELECT COUNT(*)::INTEGER INTO test_result FROM assets;
    RETURN QUERY SELECT 
        'Yasin Context'::TEXT,
        'Yasin Tenant'::TEXT,
        'assets'::TEXT,
        1::INTEGER, -- Expected: 1 asset (Yasin's logo)
        test_result.count::INTEGER,
        CASE WHEN test_result.count = 1 THEN '✅ PASS' ELSE '❌ FAIL' END::TEXT;
    
    -- Test content table (legacy data)
    SELECT COUNT(*)::INTEGER INTO test_result FROM content;
    RETURN QUERY SELECT 
        'Yasin Context'::TEXT,
        'Yasin Tenant'::TEXT,
        'content'::TEXT,
        2::INTEGER, -- Expected: 2 content items (Yasin's articles)
        test_result.count::INTEGER,
        CASE WHEN test_result.count = 2 THEN '✅ PASS' ELSE '❌ FAIL' END::TEXT;
    
    -- Test 2: Joey's context - should only see Joey's data
    \echo '👤 Testing Joey''s tenant context...'
    
    -- Set Joey's tenant context
    PERFORM set_config('rls.tenant_id', joey_tenant_id::text, true);
    
    -- Test sites table
    SELECT COUNT(*)::INTEGER INTO test_result FROM sites;
    RETURN QUERY SELECT 
        'Joey Context'::TEXT,
        'Joey Tenant'::TEXT,
        'sites'::TEXT,
        1::INTEGER, -- Expected: 1 site (Joey's)
        test_result.count::INTEGER,
        CASE WHEN test_result.count = 1 THEN '✅ PASS' ELSE '❌ FAIL' END::TEXT;
    
    -- Test pages table
    SELECT COUNT(*)::INTEGER INTO test_result FROM pages;
    RETURN QUERY SELECT 
        'Joey Context'::TEXT,
        'Joey Tenant'::TEXT,
        'pages'::TEXT,
        2::INTEGER, -- Expected: 2 pages (Joey's home + about)
        test_result.count::INTEGER,
        CASE WHEN test_result.count = 2 THEN '✅ PASS' ELSE '❌ FAIL' END::TEXT;
    
    -- Test assets table
    SELECT COUNT(*)::INTEGER INTO test_result FROM assets;
    RETURN QUERY SELECT 
        'Joey Context'::TEXT,
        'Joey Tenant'::TEXT,
        'assets'::TEXT,
        1::INTEGER, -- Expected: 1 asset (Joey's banner)
        test_result.count::INTEGER,
        CASE WHEN test_result.count = 1 THEN '✅ PASS' ELSE '❌ FAIL' END::TEXT;
    
    -- Test content table (legacy data)
    SELECT COUNT(*)::INTEGER INTO test_result FROM content;
    RETURN QUERY SELECT 
        'Joey Context'::TEXT,
        'Joey Tenant'::TEXT,
        'content'::TEXT,
        3::INTEGER, -- Expected: 3 content items (Joey's articles)
        test_result.count::INTEGER,
        CASE WHEN test_result.count = 3 THEN '✅ PASS' ELSE '❌ FAIL' END::TEXT;
    
    -- Test 3: No context - should see nothing (or fail safely)
    \echo '🚫 Testing no tenant context...'
    
    -- Clear tenant context
    PERFORM set_config('rls.tenant_id', '', true);
    
    -- Test sites table
    SELECT COUNT(*)::INTEGER INTO test_result FROM sites;
    RETURN QUERY SELECT 
        'No Context'::TEXT,
        'None'::TEXT,
        'sites'::TEXT,
        0::INTEGER, -- Expected: 0 sites (no access)
        test_result.count::INTEGER,
        CASE WHEN test_result.count = 0 THEN '✅ PASS' ELSE '❌ FAIL' END::TEXT;
    
    -- Test pages table
    SELECT COUNT(*)::INTEGER INTO test_result FROM pages;
    RETURN QUERY SELECT 
        'No Context'::TEXT,
        'None'::TEXT,
        'pages'::TEXT,
        0::INTEGER, -- Expected: 0 pages (no access)
        test_result.count::INTEGER,
        CASE WHEN test_result.count = 0 THEN '✅ PASS' ELSE '❌ FAIL' END::TEXT;
    
    -- Test assets table
    SELECT COUNT(*)::INTEGER INTO test_result FROM assets;
    RETURN QUERY SELECT 
        'No Context'::TEXT,
        'None'::TEXT,
        'assets'::TEXT,
        0::INTEGER, -- Expected: 0 assets (no access)
        test_result.count::INTEGER,
        CASE WHEN test_result.count = 0 THEN '✅ PASS' ELSE '❌ FAIL' END::TEXT;

END;
$$ LANGUAGE plpgsql;

-- Function to test cross-tenant access attempts
CREATE OR REPLACE FUNCTION test_cross_tenant_access()
RETURNS TABLE(
    test_name TEXT,
    operation TEXT,
    target_resource TEXT,
    status TEXT,
    details TEXT
) AS $$
DECLARE
    yasin_tenant_id UUID := '11111111-1111-1111-1111-111111111111';
    joey_tenant_id UUID := '22222222-2222-2222-2222-222222222222';
    yasin_site_id UUID := 'test0001-site-yasin-0000-000000000001';
    joey_site_id UUID := 'test0002-site-joey-0000-000000000002';
    result_count INTEGER;
BEGIN
    \echo '🔐 Testing Cross-Tenant Access Prevention...'
    \echo ''
    
    -- Test 1: Yasin trying to access Joey's site
    PERFORM set_config('rls.tenant_id', yasin_tenant_id::text, true);
    
    SELECT COUNT(*) INTO result_count FROM sites WHERE id = joey_site_id;
    RETURN QUERY SELECT 
        'Cross-Tenant Read'::TEXT,
        'SELECT'::TEXT,
        'Joey''s site from Yasin context'::TEXT,
        CASE WHEN result_count = 0 THEN '✅ BLOCKED' ELSE '❌ LEAKED' END::TEXT,
        format('Found %s records (should be 0)', result_count)::TEXT;
    
    -- Test 2: Joey trying to access Yasin's pages
    PERFORM set_config('rls.tenant_id', joey_tenant_id::text, true);
    
    SELECT COUNT(*) INTO result_count FROM pages WHERE site_id = yasin_site_id;
    RETURN QUERY SELECT 
        'Cross-Tenant Read'::TEXT,
        'SELECT'::TEXT,
        'Yasin''s pages from Joey context'::TEXT,
        CASE WHEN result_count = 0 THEN '✅ BLOCKED' ELSE '❌ LEAKED' END::TEXT,
        format('Found %s records (should be 0)', result_count)::TEXT;
    
    -- Test 3: Attempt to insert into wrong tenant
    BEGIN
        PERFORM set_config('rls.tenant_id', yasin_tenant_id::text, true);
        INSERT INTO sites (tenant_id, name, subdomain) 
        VALUES (joey_tenant_id, 'Malicious Site', 'malicious-site');
        
        RETURN QUERY SELECT 
            'Cross-Tenant Write'::TEXT,
            'INSERT'::TEXT,
            'Site with wrong tenant_id'::TEXT,
            '❌ ALLOWED'::TEXT,
            'Insert succeeded - RLS policy may be weak'::TEXT;
    EXCEPTION WHEN OTHERS THEN
        RETURN QUERY SELECT 
            'Cross-Tenant Write'::TEXT,
            'INSERT'::TEXT,
            'Site with wrong tenant_id'::TEXT,
            '✅ BLOCKED'::TEXT,
            format('Insert failed: %s', SQLERRM)::TEXT;
    END;
    
    -- Test 4: Attempt to update another tenant's data
    BEGIN
        PERFORM set_config('rls.tenant_id', yasin_tenant_id::text, true);
        UPDATE sites SET name = 'Hacked Site' WHERE id = joey_site_id;
        
        GET DIAGNOSTICS result_count = ROW_COUNT;
        RETURN QUERY SELECT 
            'Cross-Tenant Write'::TEXT,
            'UPDATE'::TEXT,
            'Joey''s site from Yasin context'::TEXT,
            CASE WHEN result_count = 0 THEN '✅ BLOCKED' ELSE '❌ ALLOWED' END::TEXT,
            format('Updated %s records (should be 0)', result_count)::TEXT;
    EXCEPTION WHEN OTHERS THEN
        RETURN QUERY SELECT 
            'Cross-Tenant Write'::TEXT,
            'UPDATE'::TEXT,
            'Joey''s site from Yasin context'::TEXT,
            '✅ BLOCKED'::TEXT,
            format('Update failed: %s', SQLERRM)::TEXT;
    END;

END;
$$ LANGUAGE plpgsql;

-- Function to check RLS status on all tables
CREATE OR REPLACE FUNCTION check_rls_status()
RETURNS TABLE(
    table_name TEXT,
    rls_enabled BOOLEAN,
    force_rls BOOLEAN,
    policy_count INTEGER,
    status TEXT
) AS $$
BEGIN
    RETURN QUERY
    SELECT 
        t.tablename::TEXT,
        t.rowsecurity,
        COALESCE(t.forcerowsecurity, false),
        (SELECT COUNT(*) FROM pg_policies p WHERE p.tablename = t.tablename)::INTEGER,
        CASE 
            WHEN t.rowsecurity AND (SELECT COUNT(*) FROM pg_policies p WHERE p.tablename = t.tablename) > 0 
            THEN '✅ PROTECTED'
            WHEN t.rowsecurity AND (SELECT COUNT(*) FROM pg_policies p WHERE p.tablename = t.tablename) = 0
            THEN '⚠️  RLS ON, NO POLICIES'
            WHEN NOT t.rowsecurity
            THEN '❌ UNPROTECTED'
            ELSE '❓ UNKNOWN'
        END::TEXT
    FROM pg_tables t
    WHERE t.schemaname = 'public' 
    AND t.tablename IN ('tenants', 'users', 'content', 'sites', 'pages', 'templates', 'assets', 'domains', 'widgets')
    ORDER BY t.tablename;
END;
$$ LANGUAGE plpgsql;

-- Run all tests
\echo '📊 RLS Status Check'
\echo '==================='
SELECT * FROM check_rls_status();

\echo ''
\echo '🧪 RLS Isolation Tests'
\echo '======================'
SELECT * FROM test_rls_isolation();

\echo ''
\echo '🔐 Cross-Tenant Access Tests'
\echo '============================='
SELECT * FROM test_cross_tenant_access();

-- Additional verification queries
\echo ''
\echo '🔍 Additional Verification'
\echo '=========================='

-- Show current RLS context
\echo 'Current RLS context:'
SELECT 
    current_setting('rls.tenant_id', true) as current_tenant_id,
    current_setting('rls.user_id', true) as current_user_id;

-- Show total data counts without RLS context
PERFORM set_config('rls.tenant_id', '', true);
\echo ''
\echo 'Total data counts (no RLS context):'
SELECT 
    'sites' as table_name, COUNT(*) as total_records FROM sites
UNION ALL
SELECT 
    'pages' as table_name, COUNT(*) as total_records FROM pages
UNION ALL
SELECT 
    'assets' as table_name, COUNT(*) as total_records FROM assets
UNION ALL
SELECT 
    'content' as table_name, COUNT(*) as total_records FROM content
ORDER BY table_name;

-- Test with Yasin's context
PERFORM set_config('rls.tenant_id', '11111111-1111-1111-1111-111111111111'::text, true);
\echo ''
\echo 'Data visible to Yasin (tenant: 11111111-1111-1111-1111-111111111111):'
SELECT 
    'sites' as table_name, COUNT(*) as visible_records FROM sites
UNION ALL
SELECT 
    'pages' as table_name, COUNT(*) as visible_records FROM pages
UNION ALL
SELECT 
    'assets' as table_name, COUNT(*) as visible_records FROM assets
UNION ALL
SELECT 
    'content' as table_name, COUNT(*) as visible_records FROM content
ORDER BY table_name;

-- Test with Joey's context
PERFORM set_config('rls.tenant_id', '22222222-2222-2222-2222-222222222222'::text, true);
\echo ''
\echo 'Data visible to Joey (tenant: 22222222-2222-2222-2222-222222222222):'
SELECT 
    'sites' as table_name, COUNT(*) as visible_records FROM sites
UNION ALL
SELECT 
    'pages' as table_name, COUNT(*) as visible_records FROM pages
UNION ALL
SELECT 
    'assets' as table_name, COUNT(*) as visible_records FROM assets
UNION ALL
SELECT 
    'content' as table_name, COUNT(*) as visible_records FROM content
ORDER BY table_name;

\echo ''
\echo '🎯 RLS Testing Complete!'
\echo '========================'
\echo 'Review the results above to verify tenant isolation is working correctly.'
\echo 'All tests marked with ✅ indicate proper RLS functionality.'
\echo 'Any tests marked with ❌ indicate potential security vulnerabilities.'
\echo ''

-- Clean up test functions
DROP FUNCTION IF EXISTS test_rls_isolation();
DROP FUNCTION IF EXISTS test_cross_tenant_access();
DROP FUNCTION IF EXISTS check_rls_status();
