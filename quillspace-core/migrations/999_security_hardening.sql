-- ============================================================================
-- 999_security_hardening.sql
-- Permanent Security Hardening for QuillSpace Multi-Tenant Database
-- 
-- This migration ensures all security configurations persist across container restarts
-- ============================================================================

-- In PostgreSQL 15+, custom configuration parameters are allowed by default
-- No need to configure custom_variable_classes
-- Just ensure we can set session variables with proper prefixes

-- Ensure all tables have proper ownership (postgres user)
ALTER TABLE users OWNER TO postgres;
ALTER TABLE tenants OWNER TO postgres;
ALTER TABLE sites OWNER TO postgres;
ALTER TABLE pages OWNER TO postgres;
ALTER TABLE templates OWNER TO postgres;
ALTER TABLE assets OWNER TO postgres;
ALTER TABLE content OWNER TO postgres;
ALTER TABLE domains OWNER TO postgres;
ALTER TABLE analytics_events OWNER TO postgres;
ALTER TABLE user_sessions OWNER TO postgres;
ALTER TABLE audit_logs OWNER TO postgres;
ALTER TABLE casbin_rules OWNER TO postgres;
ALTER TABLE schema_migrations OWNER TO postgres;

-- Ensure RLS is enabled and forced on all tables
ALTER TABLE users ENABLE ROW LEVEL SECURITY;
ALTER TABLE users FORCE ROW LEVEL SECURITY;

ALTER TABLE tenants ENABLE ROW LEVEL SECURITY;
ALTER TABLE tenants FORCE ROW LEVEL SECURITY;

ALTER TABLE sites ENABLE ROW LEVEL SECURITY;
ALTER TABLE sites FORCE ROW LEVEL SECURITY;

ALTER TABLE pages ENABLE ROW LEVEL SECURITY;
ALTER TABLE pages FORCE ROW LEVEL SECURITY;

ALTER TABLE templates ENABLE ROW LEVEL SECURITY;
ALTER TABLE templates FORCE ROW LEVEL SECURITY;

ALTER TABLE assets ENABLE ROW LEVEL SECURITY;
ALTER TABLE assets FORCE ROW LEVEL SECURITY;

ALTER TABLE content ENABLE ROW LEVEL SECURITY;
ALTER TABLE content FORCE ROW LEVEL SECURITY;

ALTER TABLE domains ENABLE ROW LEVEL SECURITY;
ALTER TABLE domains FORCE ROW LEVEL SECURITY;

ALTER TABLE analytics_events ENABLE ROW LEVEL SECURITY;
ALTER TABLE analytics_events FORCE ROW LEVEL SECURITY;

ALTER TABLE user_sessions ENABLE ROW LEVEL SECURITY;
ALTER TABLE user_sessions FORCE ROW LEVEL SECURITY;

ALTER TABLE audit_logs ENABLE ROW LEVEL SECURITY;
ALTER TABLE audit_logs FORCE ROW LEVEL SECURITY;

ALTER TABLE casbin_rules ENABLE ROW LEVEL SECURITY;
ALTER TABLE casbin_rules FORCE ROW LEVEL SECURITY;

-- Ensure quillspace user has restricted privileges (no superuser, no RLS bypass)
ALTER USER quillspace NOSUPERUSER;
ALTER USER quillspace NOBYPASSRLS;

-- Grant necessary permissions to quillspace user
GRANT SELECT, INSERT, UPDATE, DELETE ON ALL TABLES IN SCHEMA public TO quillspace;
GRANT USAGE, SELECT ON ALL SEQUENCES IN SCHEMA public TO quillspace;

-- Create helper function to safely set session variables
CREATE OR REPLACE FUNCTION set_session_variable(var_name text, var_value text)
RETURNS void
LANGUAGE plpgsql
SECURITY DEFINER
AS $$
BEGIN
    -- Validate variable names to prevent injection
    IF var_name NOT IN ('app.current_tenant_id', 'rls.user_id', 'rls.tenant_id') THEN
        RAISE EXCEPTION 'Invalid session variable name: %', var_name;
    END IF;
    
    -- Set the session variable
    PERFORM set_config(var_name, var_value, false);
END;
$$;

-- Grant execute permission on helper function
GRANT EXECUTE ON FUNCTION set_session_variable(text, text) TO quillspace;

-- Log completion
INSERT INTO audit_logs (
    id, 
    tenant_id, 
    user_id, 
    action, 
    resource_type, 
    resource_id, 
    details, 
    created_at
) VALUES (
    gen_random_uuid(),
    '00000000-0000-0000-0000-000000000000'::uuid,
    '00000000-0000-0000-0000-000000000000'::uuid,
    'SECURITY_HARDENING',
    'DATABASE',
    'postgresql',
    '{"migration": "999_security_hardening", "status": "completed", "timestamp": "' || NOW()::text || '"}',
    NOW()
);

-- Verify security configuration
DO $$
DECLARE
    table_count INTEGER;
    rls_count INTEGER;
BEGIN
    -- Count total tables
    SELECT COUNT(*) INTO table_count 
    FROM information_schema.tables 
    WHERE table_schema = 'public' 
    AND table_type = 'BASE TABLE'
    AND table_name NOT LIKE 'pg_%';
    
    -- Count tables with RLS enabled
    SELECT COUNT(*) INTO rls_count
    FROM pg_class c
    JOIN pg_namespace n ON c.relnamespace = n.oid
    WHERE n.nspname = 'public'
    AND c.relkind = 'r'
    AND c.relrowsecurity = true;
    
    RAISE NOTICE 'Security hardening completed: % tables, % with RLS enabled', table_count, rls_count;
    
    IF rls_count < table_count THEN
        RAISE WARNING 'Not all tables have RLS enabled: %/% tables secured', rls_count, table_count;
    END IF;
END;
$$;
