-- QuillSpace Security Vulnerability Fix Script
-- This script fixes ALL identified security issues

\echo '🔧 FIXING ALL SECURITY VULNERABILITIES'
\echo '======================================'
\echo ''

-- STEP 1: Transfer ownership of ALL tables from quillspace to postgres
\echo '1️⃣ Transferring table ownership to postgres...'

ALTER TABLE tenants OWNER TO postgres;
ALTER TABLE content_categories OWNER TO postgres;
ALTER TABLE files OWNER TO postgres;
ALTER TABLE tenant_domains OWNER TO postgres;
ALTER TABLE audit_logs OWNER TO postgres;
ALTER TABLE template_versions OWNER TO postgres;
ALTER TABLE sites OWNER TO postgres;
ALTER TABLE pages OWNER TO postgres;
ALTER TABLE site_builds OWNER TO postgres;
ALTER TABLE users OWNER TO postgres;
ALTER TABLE domains OWNER TO postgres;
ALTER TABLE templates OWNER TO postgres;
ALTER TABLE widgets OWNER TO postgres;
ALTER TABLE page_analytics OWNER TO postgres;
ALTER TABLE assets OWNER TO postgres;
-- content already owned by postgres

\echo '✅ Table ownership transferred'

-- STEP 2: Re-enable and FORCE RLS on all critical tables
\echo '2️⃣ Re-enabling and forcing RLS on all tables...'

-- Re-enable RLS on users (was disabled during bypass test)
ALTER TABLE users ENABLE ROW LEVEL SECURITY;
ALTER TABLE users FORCE ROW LEVEL SECURITY;

-- Force RLS on all other critical tables
ALTER TABLE sites FORCE ROW LEVEL SECURITY;
ALTER TABLE pages FORCE ROW LEVEL SECURITY;
ALTER TABLE templates FORCE ROW LEVEL SECURITY;
ALTER TABLE assets FORCE ROW LEVEL SECURITY;
ALTER TABLE domains FORCE ROW LEVEL SECURITY;

\echo '✅ RLS forced on all critical tables'

-- STEP 3: Revoke dangerous privileges and grant only necessary ones
\echo '3️⃣ Fixing user privileges...'

-- Remove all privileges from quillspace user
REVOKE ALL PRIVILEGES ON ALL TABLES IN SCHEMA public FROM quillspace;
REVOKE ALL PRIVILEGES ON ALL SEQUENCES IN SCHEMA public FROM quillspace;
REVOKE ALL PRIVILEGES ON ALL FUNCTIONS IN SCHEMA public FROM quillspace;

-- Grant only necessary data manipulation privileges
GRANT SELECT, INSERT, UPDATE, DELETE ON ALL TABLES IN SCHEMA public TO quillspace;
GRANT USAGE, SELECT ON ALL SEQUENCES IN SCHEMA public TO quillspace;
GRANT EXECUTE ON ALL FUNCTIONS IN SCHEMA public TO quillspace;

\echo '✅ User privileges restricted'

-- STEP 4: Verify security fixes
\echo '4️⃣ Verifying security fixes...'

\echo 'Table ownership status:'
SELECT 
    tablename,
    tableowner,
    CASE 
        WHEN tableowner = 'postgres' THEN '✅ SECURE'
        ELSE '❌ STILL VULNERABLE'
    END as status
FROM pg_tables 
WHERE schemaname = 'public' 
AND tablename IN ('users', 'content', 'sites', 'pages', 'assets', 'templates')
ORDER BY tablename;

\echo ''
\echo 'RLS status:'
SELECT 
    tablename,
    rowsecurity as rls_enabled,
    CASE 
        WHEN rowsecurity = true THEN '✅ PROTECTED'
        ELSE '❌ UNPROTECTED'
    END as status
FROM pg_tables 
WHERE schemaname = 'public' 
AND tablename IN ('users', 'content', 'sites', 'pages', 'assets', 'templates')
ORDER BY tablename;

\echo ''
\echo '🎯 SECURITY FIXES COMPLETE!'
\echo '=========================='
