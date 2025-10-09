-- Configurable User Isolation System
-- Allows tenant owners to configure user-level data isolation within their tenant

-- 1. Add isolation configuration to tenants table
ALTER TABLE tenants ADD COLUMN IF NOT EXISTS user_isolation_mode VARCHAR(20) DEFAULT 'collaborative' 
    CHECK (user_isolation_mode IN ('collaborative', 'isolated', 'role_based'));

-- Add isolation settings to tenant settings
UPDATE tenants SET settings = settings || jsonb_build_object(
    'user_isolation', jsonb_build_object(
        'mode', 'collaborative',
        'allow_cross_user_read', true,
        'allow_cross_user_write', false,
        'admin_can_see_all', true
    )
) WHERE settings->>'user_isolation' IS NULL;

-- 2. Create user isolation helper functions
CREATE OR REPLACE FUNCTION get_tenant_isolation_mode(target_tenant_id UUID DEFAULT NULL)
RETURNS TEXT AS $$
DECLARE
    tenant_id_to_check UUID;
    isolation_mode TEXT;
BEGIN
    -- Use provided tenant_id or current tenant context
    tenant_id_to_check := COALESCE(target_tenant_id, current_tenant_id());
    
    IF tenant_id_to_check IS NULL THEN
        RETURN 'isolated'; -- Default to most restrictive if no context
    END IF;
    
    SELECT COALESCE(user_isolation_mode, 'collaborative') 
    INTO isolation_mode
    FROM tenants 
    WHERE id = tenant_id_to_check;
    
    RETURN COALESCE(isolation_mode, 'collaborative');
END;
$$ LANGUAGE plpgsql SECURITY DEFINER;

CREATE OR REPLACE FUNCTION current_user_id()
RETURNS UUID AS $$
BEGIN
    RETURN current_setting('rls.user_id', true)::UUID;
EXCEPTION
    WHEN OTHERS THEN
        RETURN NULL;
END;
$$ LANGUAGE plpgsql SECURITY DEFINER;

CREATE OR REPLACE FUNCTION user_can_access_resource(resource_owner_id UUID, resource_tenant_id UUID DEFAULT NULL)
RETURNS BOOLEAN AS $$
DECLARE
    current_user_uuid UUID;
    current_tenant_uuid UUID;
    isolation_mode TEXT;
    current_user_role TEXT;
BEGIN
    -- Get current context
    current_user_uuid := current_user_id();
    current_tenant_uuid := COALESCE(resource_tenant_id, current_tenant_id());
    
    -- If no user context, deny access
    IF current_user_uuid IS NULL OR current_tenant_uuid IS NULL THEN
        RETURN FALSE;
    END IF;
    
    -- If user owns the resource, always allow
    IF current_user_uuid = resource_owner_id THEN
        RETURN TRUE;
    END IF;
    
    -- Get tenant isolation mode
    isolation_mode := get_tenant_isolation_mode(current_tenant_uuid);
    
    -- Get current user role
    SELECT role INTO current_user_role 
    FROM users 
    WHERE id = current_user_uuid AND tenant_id = current_tenant_uuid;
    
    -- Apply isolation rules
    CASE isolation_mode
        WHEN 'collaborative' THEN
            -- All users in tenant can see each other's data
            RETURN TRUE;
        WHEN 'isolated' THEN
            -- Users can only see their own data (except admins)
            RETURN (current_user_role = 'admin');
        WHEN 'role_based' THEN
            -- Admins see all, editors see published content, authors see own content
            RETURN (
                current_user_role = 'admin' OR
                (current_user_role = 'editor' AND EXISTS (
                    SELECT 1 FROM content WHERE author_id = resource_owner_id AND status = 'published'
                ))
            );
        ELSE
            -- Default to most restrictive
            RETURN FALSE;
    END CASE;
END;
$$ LANGUAGE plpgsql SECURITY DEFINER;

-- 3. Update RLS policies to support configurable user isolation

-- Drop existing policies that need to be updated
DROP POLICY IF EXISTS tenant_isolation_users ON users;
DROP POLICY IF EXISTS tenant_isolation_content ON content;
DROP POLICY IF EXISTS tenant_isolation_sites ON sites;

-- Create new configurable policies for users table
CREATE POLICY configurable_user_isolation_users ON users
    FOR ALL
    USING (
        tenant_id = current_tenant_id() AND (
            -- Always allow users to see themselves
            id = current_user_id() OR
            -- Apply tenant isolation rules for seeing other users
            (get_tenant_isolation_mode() = 'collaborative') OR
            (get_tenant_isolation_mode() = 'isolated' AND EXISTS (
                SELECT 1 FROM users cu WHERE cu.id = current_user_id() AND cu.role = 'admin'
            )) OR
            (get_tenant_isolation_mode() = 'role_based' AND EXISTS (
                SELECT 1 FROM users cu WHERE cu.id = current_user_id() AND cu.role IN ('admin', 'editor')
            ))
        )
    )
    WITH CHECK (
        tenant_id = current_tenant_id() AND (
            id = current_user_id() OR
            EXISTS (SELECT 1 FROM users cu WHERE cu.id = current_user_id() AND cu.role = 'admin')
        )
    );

-- Create configurable policy for content table
CREATE POLICY configurable_user_isolation_content ON content
    FOR ALL
    USING (
        tenant_id = current_tenant_id() AND (
            -- Owner can always see their content
            author_id = current_user_id() OR
            -- Apply tenant isolation rules
            user_can_access_resource(author_id, tenant_id)
        )
    )
    WITH CHECK (
        tenant_id = current_tenant_id() AND (
            -- Users can create content as themselves
            author_id = current_user_id() OR
            -- Admins can create content for others
            EXISTS (SELECT 1 FROM users cu WHERE cu.id = current_user_id() AND cu.role = 'admin')
        )
    );

-- Create configurable policy for sites table  
CREATE POLICY configurable_user_isolation_sites ON sites
    FOR ALL
    USING (
        tenant_id = current_tenant_id() AND (
            -- If sites have owner, check ownership; otherwise use collaborative model
            NOT EXISTS (SELECT 1 FROM information_schema.columns 
                       WHERE table_name = 'sites' AND column_name = 'owner_id') OR
            get_tenant_isolation_mode() = 'collaborative' OR
            EXISTS (SELECT 1 FROM users cu WHERE cu.id = current_user_id() AND cu.role = 'admin')
        )
    )
    WITH CHECK (
        tenant_id = current_tenant_id()
    );

-- 4. Create tenant isolation management functions for tenant admins

CREATE OR REPLACE FUNCTION set_tenant_user_isolation(
    isolation_mode TEXT,
    allow_cross_user_read BOOLEAN DEFAULT NULL,
    allow_cross_user_write BOOLEAN DEFAULT NULL
)
RETURNS TEXT AS $$
DECLARE
    current_tenant_uuid UUID;
    current_user_uuid UUID;
    is_admin BOOLEAN;
BEGIN
    -- Verify user is admin of current tenant
    current_tenant_uuid := current_tenant_id();
    current_user_uuid := current_user_id();
    
    IF current_tenant_uuid IS NULL OR current_user_uuid IS NULL THEN
        RAISE EXCEPTION 'No tenant or user context set';
    END IF;
    
    SELECT (role = 'admin') INTO is_admin
    FROM users 
    WHERE id = current_user_uuid AND tenant_id = current_tenant_uuid;
    
    IF NOT is_admin THEN
        RAISE EXCEPTION 'Only tenant admins can modify isolation settings';
    END IF;
    
    -- Validate isolation mode
    IF isolation_mode NOT IN ('collaborative', 'isolated', 'role_based') THEN
        RAISE EXCEPTION 'Invalid isolation mode. Must be: collaborative, isolated, or role_based';
    END IF;
    
    -- Update tenant settings
    UPDATE tenants 
    SET 
        user_isolation_mode = isolation_mode,
        settings = settings || jsonb_build_object(
            'user_isolation', jsonb_build_object(
                'mode', isolation_mode,
                'allow_cross_user_read', COALESCE(allow_cross_user_read, true),
                'allow_cross_user_write', COALESCE(allow_cross_user_write, false),
                'updated_at', extract(epoch from now()),
                'updated_by', current_user_uuid
            )
        )
    WHERE id = current_tenant_uuid;
    
    RETURN format('Tenant isolation mode updated to: %s', isolation_mode);
END;
$$ LANGUAGE plpgsql SECURITY DEFINER;

-- 5. Create view for tenant admins to see isolation settings
CREATE OR REPLACE VIEW tenant_isolation_settings AS
SELECT 
    t.id as tenant_id,
    t.name as tenant_name,
    t.user_isolation_mode,
    t.settings->'user_isolation' as isolation_config,
    COUNT(u.id) as total_users,
    COUNT(CASE WHEN u.role = 'admin' THEN 1 END) as admin_users,
    COUNT(CASE WHEN u.role = 'editor' THEN 1 END) as editor_users,
    COUNT(CASE WHEN u.role = 'author' THEN 1 END) as author_users
FROM tenants t
LEFT JOIN users u ON t.id = u.tenant_id
WHERE t.id = current_tenant_id()
GROUP BY t.id, t.name, t.user_isolation_mode, t.settings;

-- Grant permissions
GRANT SELECT ON tenant_isolation_settings TO quillspace, quillspace_admin;
GRANT EXECUTE ON FUNCTION set_tenant_user_isolation(TEXT, BOOLEAN, BOOLEAN) TO quillspace;
GRANT EXECUTE ON FUNCTION get_tenant_isolation_mode(UUID) TO quillspace, quillspace_admin;
GRANT EXECUTE ON FUNCTION user_can_access_resource(UUID, UUID) TO quillspace, quillspace_admin;

-- 6. Display implementation summary
DO $$
BEGIN
    RAISE NOTICE '🔧 CONFIGURABLE USER ISOLATION IMPLEMENTED';
    RAISE NOTICE '==========================================';
    RAISE NOTICE '';
    RAISE NOTICE '📋 ISOLATION MODES:';
    RAISE NOTICE '• collaborative: All users see each others data (default)';
    RAISE NOTICE '• isolated: Users only see own data (admins see all)';
    RAISE NOTICE '• role_based: Admins see all, editors see published, authors see own';
    RAISE NOTICE '';
    RAISE NOTICE '🔧 MANAGEMENT FUNCTIONS:';
    RAISE NOTICE '• set_tenant_user_isolation(mode) - Change isolation mode';
    RAISE NOTICE '• get_tenant_isolation_mode() - Check current mode';
    RAISE NOTICE '• tenant_isolation_settings VIEW - See current settings';
    RAISE NOTICE '';
    RAISE NOTICE '🔒 SECURITY FEATURES:';
    RAISE NOTICE '• Only tenant admins can change isolation settings';
    RAISE NOTICE '• RLS policies automatically enforce chosen mode';
    RAISE NOTICE '• User context (rls.user_id) required for user-level isolation';
    RAISE NOTICE '• Tenant context (app.current_tenant_id) required for tenant isolation';
END $$;
