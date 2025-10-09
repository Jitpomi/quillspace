-- Admin User Security Hardening
-- This script secures admin access and creates proper admin user hierarchy

-- 1. Create audit table for admin actions
CREATE TABLE IF NOT EXISTS admin_audit_log (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    admin_user TEXT NOT NULL,
    action TEXT NOT NULL,
    target_table TEXT,
    target_tenant_id UUID,
    query_text TEXT,
    timestamp TIMESTAMPTZ DEFAULT NOW(),
    client_addr INET,
    session_id TEXT
);

-- 2. Create admin action logging function
CREATE OR REPLACE FUNCTION log_admin_action()
RETURNS EVENT_TRIGGER AS $$
DECLARE
    audit_query TEXT;
BEGIN
    -- Log DDL commands from admin users
    IF current_user IN ('postgres', 'quillspace_admin') THEN
        INSERT INTO admin_audit_log (admin_user, action, query_text, session_id)
        VALUES (
            current_user,
            tg_tag,
            current_query(),
            to_hex(hashtext(session_user || extract(epoch from now())::text))
        );
    END IF;
END;
$$ LANGUAGE plpgsql;

-- 3. Create event trigger for admin auditing
DROP EVENT TRIGGER IF EXISTS admin_ddl_audit;
CREATE EVENT TRIGGER admin_ddl_audit
    ON ddl_command_end
    EXECUTE FUNCTION log_admin_action();

-- 4. Create secure admin user (if not exists)
DO $$
BEGIN
    IF NOT EXISTS (SELECT FROM pg_roles WHERE rolname = 'quillspace_admin') THEN
        CREATE USER quillspace_admin WITH PASSWORD 'CHANGE_THIS_PASSWORD_IN_PRODUCTION';
        
        -- Grant necessary privileges
        GRANT SELECT, INSERT, UPDATE, DELETE ON ALL TABLES IN SCHEMA public TO quillspace_admin;
        GRANT USAGE, SELECT ON ALL SEQUENCES IN SCHEMA public TO quillspace_admin;
        GRANT EXECUTE ON ALL FUNCTIONS IN SCHEMA public TO quillspace_admin;
        
        -- Allow admin to read audit logs
        GRANT SELECT ON admin_audit_log TO quillspace_admin;
    END IF;
END $$;

-- 5. Secure postgres superuser (recommendations)
-- NOTE: These are recommendations - implement based on your security requirements

-- Create view for admin to safely check cross-tenant data
CREATE OR REPLACE VIEW admin_tenant_summary AS
SELECT 
    t.id as tenant_id,
    t.name as tenant_name,
    COUNT(DISTINCT u.id) as user_count,
    COUNT(DISTINCT c.id) as content_count,
    COUNT(DISTINCT s.id) as site_count
FROM tenants t
LEFT JOIN users u ON t.id = u.tenant_id
LEFT JOIN content c ON t.id = c.tenant_id  
LEFT JOIN sites s ON t.id = s.tenant_id
GROUP BY t.id, t.name
ORDER BY t.name;

-- Grant access to summary view
GRANT SELECT ON admin_tenant_summary TO quillspace_admin;

-- 6. Create admin helper functions
CREATE OR REPLACE FUNCTION admin_set_tenant_context(target_tenant_id UUID)
RETURNS TEXT AS $$
BEGIN
    -- Only allow admin users to set arbitrary tenant contexts
    IF current_user NOT IN ('postgres', 'quillspace_admin') THEN
        RAISE EXCEPTION 'Access denied: Only admin users can set arbitrary tenant contexts';
    END IF;
    
    -- Log the context change
    INSERT INTO admin_audit_log (admin_user, action, target_tenant_id)
    VALUES (current_user, 'SET_TENANT_CONTEXT', target_tenant_id);
    
    -- Set the context
    PERFORM set_config('app.current_tenant_id', target_tenant_id::text, true);
    
    RETURN format('Tenant context set to: %s', target_tenant_id);
END;
$$ LANGUAGE plpgsql SECURITY DEFINER;

-- Grant execute to admin
GRANT EXECUTE ON FUNCTION admin_set_tenant_context(UUID) TO quillspace_admin;

-- 7. Display admin security status
DO $$
DECLARE
    postgres_super BOOLEAN;
    admin_super BOOLEAN;
    app_super BOOLEAN;
BEGIN
    SELECT rolsuper INTO postgres_super FROM pg_roles WHERE rolname = 'postgres';
    SELECT rolsuper INTO admin_super FROM pg_roles WHERE rolname = 'quillspace_admin';
    SELECT rolsuper INTO app_super FROM pg_roles WHERE rolname = 'quillspace';
    
    RAISE NOTICE '🔐 ADMIN SECURITY STATUS';
    RAISE NOTICE '========================';
    RAISE NOTICE 'postgres user: % (superuser: %)', 
        CASE WHEN postgres_super THEN '⚠️ SUPERUSER' ELSE '✅ LIMITED' END, postgres_super;
    RAISE NOTICE 'quillspace_admin user: % (superuser: %)', 
        CASE WHEN admin_super THEN '❌ DANGEROUS' ELSE '✅ SECURE' END, admin_super;
    RAISE NOTICE 'quillspace user: % (superuser: %)', 
        CASE WHEN app_super THEN '❌ DANGEROUS' ELSE '✅ SECURE' END, app_super;
    RAISE NOTICE '';
    RAISE NOTICE '📋 ADMIN RECOMMENDATIONS:';
    RAISE NOTICE '• Use postgres user ONLY for schema changes and emergencies';
    RAISE NOTICE '• Use quillspace_admin for routine cross-tenant operations';
    RAISE NOTICE '• Use quillspace for normal application operations';
    RAISE NOTICE '• Monitor admin_audit_log for suspicious activity';
    RAISE NOTICE '• Change default admin passwords immediately';
END $$;
