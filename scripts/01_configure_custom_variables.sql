-- ============================================================================
-- 01_configure_custom_variables.sql
-- Configure PostgreSQL to support custom session variables for RLS
-- This script runs during container initialization
-- ============================================================================

-- Test that custom configuration parameters work
DO $$
BEGIN
    -- Test setting a custom configuration parameter
    PERFORM set_config('app.test_parameter', 'test_value', false);
    
    -- Test retrieving it
    IF current_setting('app.test_parameter', true) = 'test_value' THEN
        RAISE NOTICE 'Custom configuration parameters are working correctly';
    ELSE
        RAISE EXCEPTION 'Custom configuration parameters are not working';
    END IF;
    
    -- Clean up test parameter
    PERFORM set_config('app.test_parameter', NULL, false);
END;
$$;

-- Create a helper function to safely set RLS context
CREATE OR REPLACE FUNCTION set_rls_context(tenant_id_param UUID, user_id_param UUID DEFAULT NULL)
RETURNS void
LANGUAGE plpgsql
SECURITY DEFINER
AS $$
BEGIN
    -- Set tenant context (required)
    PERFORM set_config('app.current_tenant_id', tenant_id_param::text, true);
    
    -- Set user context (optional)
    IF user_id_param IS NOT NULL THEN
        PERFORM set_config('rls.user_id', user_id_param::text, true);
    END IF;
    
    -- Log the context setting for debugging
    RAISE DEBUG 'RLS context set: tenant_id=%, user_id=%', tenant_id_param, user_id_param;
END;
$$;

-- Create a helper function to clear RLS context
CREATE OR REPLACE FUNCTION clear_rls_context()
RETURNS void
LANGUAGE plpgsql
SECURITY DEFINER
AS $$
BEGIN
    PERFORM set_config('app.current_tenant_id', NULL, true);
    PERFORM set_config('rls.user_id', NULL, true);
    RAISE DEBUG 'RLS context cleared';
END;
$$;

-- Create a helper function to get current tenant context
CREATE OR REPLACE FUNCTION get_current_tenant_id()
RETURNS UUID
LANGUAGE plpgsql
STABLE
AS $$
BEGIN
    RETURN current_setting('app.current_tenant_id', true)::UUID;
EXCEPTION
    WHEN OTHERS THEN
        RETURN NULL;
END;
$$;

-- Grant permissions on helper functions to quillspace user
GRANT EXECUTE ON FUNCTION set_rls_context(UUID, UUID) TO quillspace;
GRANT EXECUTE ON FUNCTION clear_rls_context() TO quillspace;
GRANT EXECUTE ON FUNCTION get_current_tenant_id() TO quillspace;

-- Test the helper functions
DO $$
DECLARE
    test_tenant_id UUID := '11111111-1111-1111-1111-111111111111';
    retrieved_tenant_id UUID;
BEGIN
    -- Test setting context
    PERFORM set_rls_context(test_tenant_id);
    
    -- Test retrieving context
    retrieved_tenant_id := get_current_tenant_id();
    
    IF retrieved_tenant_id = test_tenant_id THEN
        RAISE NOTICE 'RLS helper functions are working correctly';
    ELSE
        RAISE EXCEPTION 'RLS helper functions failed: expected %, got %', test_tenant_id, retrieved_tenant_id;
    END IF;
    
    -- Clean up
    PERFORM clear_rls_context();
END;
$$;

-- Log successful configuration
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
    'CONFIGURE_CUSTOM_VARIABLES',
    'POSTGRESQL',
    'session_variables',
    '{"status": "success", "timestamp": "' || NOW()::text || '", "functions_created": ["set_rls_context", "clear_rls_context", "get_current_tenant_id"]}',
    NOW()
) ON CONFLICT DO NOTHING;

RAISE NOTICE 'Custom session variables configured successfully';
