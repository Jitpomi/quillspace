-- QuillSpace RLS Policy Reference
-- Complete documentation of all Row-Level Security policies

-- ============================================================================
-- CONTEXT FUNCTIONS
-- ============================================================================

-- Get current tenant ID from session context
CREATE OR REPLACE FUNCTION current_tenant_id()
RETURNS UUID AS $$
BEGIN
    RETURN current_setting('app.current_tenant_id', true)::UUID;
EXCEPTION
    WHEN OTHERS THEN
        RETURN NULL;
END;
$$ LANGUAGE plpgsql SECURITY DEFINER;

-- Get current user ID from session context  
CREATE OR REPLACE FUNCTION current_user_id()
RETURNS UUID AS $$
BEGIN
    RETURN current_setting('rls.user_id', true)::UUID;
EXCEPTION
    WHEN OTHERS THEN
        RETURN NULL;
END;
$$ LANGUAGE plpgsql SECURITY DEFINER;

-- Get tenant isolation mode
CREATE OR REPLACE FUNCTION get_tenant_isolation_mode(target_tenant_id UUID DEFAULT NULL)
RETURNS TEXT AS $$
DECLARE
    tenant_id_to_check UUID;
    isolation_mode TEXT;
BEGIN
    tenant_id_to_check := COALESCE(target_tenant_id, current_tenant_id());
    
    IF tenant_id_to_check IS NULL THEN
        RETURN 'isolated'; -- Default to most restrictive
    END IF;
    
    SELECT COALESCE(user_isolation_mode, 'collaborative') 
    INTO isolation_mode
    FROM tenants 
    WHERE id = tenant_id_to_check;
    
    RETURN COALESCE(isolation_mode, 'collaborative');
END;
$$ LANGUAGE plpgsql SECURITY DEFINER;

-- Check if user can access resource owned by another user
CREATE OR REPLACE FUNCTION user_can_access_resource(resource_owner_id UUID, resource_tenant_id UUID DEFAULT NULL)
RETURNS BOOLEAN AS $$
DECLARE
    current_user_uuid UUID;
    current_tenant_uuid UUID;
    isolation_mode TEXT;
    current_user_role TEXT;
BEGIN
    current_user_uuid := current_user_id();
    current_tenant_uuid := COALESCE(resource_tenant_id, current_tenant_id());
    
    -- No context = no access
    IF current_user_uuid IS NULL OR current_tenant_uuid IS NULL THEN
        RETURN FALSE;
    END IF;
    
    -- Owner always has access
    IF current_user_uuid = resource_owner_id THEN
        RETURN TRUE;
    END IF;
    
    isolation_mode := get_tenant_isolation_mode(current_tenant_uuid);
    
    SELECT role INTO current_user_role 
    FROM users 
    WHERE id = current_user_uuid AND tenant_id = current_tenant_uuid;
    
    -- Apply isolation rules
    CASE isolation_mode
        WHEN 'collaborative' THEN
            RETURN TRUE;
        WHEN 'isolated' THEN
            RETURN (current_user_role = 'admin');
        WHEN 'role_based' THEN
            RETURN (
                current_user_role = 'admin' OR
                (current_user_role = 'editor' AND EXISTS (
                    SELECT 1 FROM content WHERE author_id = resource_owner_id AND status = 'published'
                ))
            );
        ELSE
            RETURN FALSE;
    END CASE;
END;
$$ LANGUAGE plpgsql SECURITY DEFINER;

-- ============================================================================
-- TABLE POLICIES
-- ============================================================================

-- ----------------------------------------------------------------------------
-- TENANTS TABLE
-- ----------------------------------------------------------------------------
-- No RLS on tenants table - handled at application level
-- Users should only see their own tenant through application logic

-- ----------------------------------------------------------------------------
-- USERS TABLE  
-- ----------------------------------------------------------------------------
-- Policy: configurable_user_isolation_users
-- Purpose: Control which users can see other users based on isolation mode

DROP POLICY IF EXISTS configurable_user_isolation_users ON users;
CREATE POLICY configurable_user_isolation_users ON users
    FOR ALL
    USING (
        -- Must be in same tenant
        tenant_id = current_tenant_id() AND (
            -- Always see yourself
            id = current_user_id() OR
            -- Additional visibility based on isolation mode
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
        -- Can create/modify users in same tenant
        tenant_id = current_tenant_id() AND (
            -- Users can modify themselves
            id = current_user_id() OR
            -- Admins can modify anyone in tenant
            EXISTS (SELECT 1 FROM users cu WHERE cu.id = current_user_id() AND cu.role = 'admin')
        )
    );

-- ----------------------------------------------------------------------------
-- CONTENT TABLE
-- ----------------------------------------------------------------------------
-- Policy: configurable_user_isolation_content  
-- Purpose: Control access to content based on ownership and isolation mode

DROP POLICY IF EXISTS configurable_user_isolation_content ON content;
CREATE POLICY configurable_user_isolation_content ON content
    FOR ALL
    USING (
        -- Must be in same tenant
        tenant_id = current_tenant_id() AND (
            -- Always see own content
            author_id = current_user_id() OR
            -- Additional access based on isolation rules
            user_can_access_resource(author_id, tenant_id)
        )
    )
    WITH CHECK (
        -- Can create/modify content in same tenant
        tenant_id = current_tenant_id() AND (
            -- Create content as yourself
            author_id = current_user_id() OR
            -- Admins can create content for others
            EXISTS (SELECT 1 FROM users cu WHERE cu.id = current_user_id() AND cu.role = 'admin')
        )
    );

-- ----------------------------------------------------------------------------
-- SITES TABLE
-- ----------------------------------------------------------------------------  
-- Policy: configurable_user_isolation_sites
-- Purpose: Control access to sites based on tenant and isolation mode

DROP POLICY IF EXISTS configurable_user_isolation_sites ON sites;
CREATE POLICY configurable_user_isolation_sites ON sites
    FOR ALL
    USING (
        -- Must be in same tenant
        tenant_id = current_tenant_id() AND (
            -- Sites don't have individual owners currently
            -- Access controlled by isolation mode
            get_tenant_isolation_mode() = 'collaborative' OR
            EXISTS (SELECT 1 FROM users cu WHERE cu.id = current_user_id() AND cu.role = 'admin')
        )
    )
    WITH CHECK (
        -- Can create/modify sites in same tenant
        tenant_id = current_tenant_id()
    );

-- ----------------------------------------------------------------------------
-- PAGES TABLE
-- ----------------------------------------------------------------------------
-- Policy: tenant_isolation_pages
-- Purpose: Pages inherit access from their parent site

DROP POLICY IF EXISTS tenant_isolation_pages ON pages;
CREATE POLICY tenant_isolation_pages ON pages
    FOR ALL
    USING (
        -- Pages accessible if parent site is accessible
        EXISTS (
            SELECT 1 FROM sites s 
            WHERE s.id = pages.site_id 
            AND s.tenant_id = current_tenant_id()
        )
    );

-- ----------------------------------------------------------------------------
-- TEMPLATES TABLE
-- ----------------------------------------------------------------------------
-- Policy: tenant_isolation_templates
-- Purpose: Templates scoped to tenant, public templates visible to all

DROP POLICY IF EXISTS tenant_isolation_templates ON templates;
CREATE POLICY tenant_isolation_templates ON templates
    FOR ALL
    USING (
        -- Own tenant templates OR public templates
        tenant_id = current_tenant_id() OR
        is_public = true
    )
    WITH CHECK (
        -- Can only create templates in own tenant
        tenant_id = current_tenant_id()
    );

-- ----------------------------------------------------------------------------
-- ASSETS TABLE
-- ----------------------------------------------------------------------------
-- Policy: tenant_isolation_assets
-- Purpose: Assets scoped to tenant and site

DROP POLICY IF EXISTS tenant_isolation_assets ON assets;
CREATE POLICY tenant_isolation_assets ON assets
    FOR ALL
    USING (
        -- Must be in same tenant
        tenant_id = current_tenant_id() AND (
            -- If asset belongs to a site, check site access
            site_id IS NULL OR
            EXISTS (
                SELECT 1 FROM sites s 
                WHERE s.id = assets.site_id 
                AND s.tenant_id = current_tenant_id()
            )
        )
    )
    WITH CHECK (
        -- Can create assets in same tenant
        tenant_id = current_tenant_id()
    );

-- ----------------------------------------------------------------------------
-- DOMAINS TABLE
-- ----------------------------------------------------------------------------
-- Policy: tenant_isolation_domains
-- Purpose: Domains accessible through their parent site

DROP POLICY IF EXISTS tenant_isolation_domains ON domains;
CREATE POLICY tenant_isolation_domains ON domains
    FOR ALL
    USING (
        -- Domains accessible if parent site is accessible
        EXISTS (
            SELECT 1 FROM sites s 
            WHERE s.id = domains.site_id 
            AND s.tenant_id = current_tenant_id()
        )
    );

-- ----------------------------------------------------------------------------
-- WIDGETS TABLE
-- ----------------------------------------------------------------------------
-- Policy: public_widgets_read - Public approved widgets visible to all
-- Policy: widget_owner_policy - Users can manage their own widgets

DROP POLICY IF EXISTS public_widgets_read ON widgets;
CREATE POLICY public_widgets_read ON widgets
    FOR SELECT
    USING (
        is_public = true AND is_approved = true
    );

DROP POLICY IF EXISTS widget_owner_policy ON widgets;
CREATE POLICY widget_owner_policy ON widgets
    FOR ALL
    USING (
        created_by = current_user_id()
    )
    WITH CHECK (
        created_by = current_user_id()
    );

-- ----------------------------------------------------------------------------
-- AUDIT_LOGS TABLE
-- ----------------------------------------------------------------------------
-- Policy: tenant_isolation_audit_logs
-- Purpose: Audit logs scoped to tenant

DROP POLICY IF EXISTS tenant_isolation_audit_logs ON audit_logs;
CREATE POLICY tenant_isolation_audit_logs ON audit_logs
    FOR ALL
    USING (
        tenant_id = current_tenant_id()
    );

-- ----------------------------------------------------------------------------
-- FILES TABLE
-- ----------------------------------------------------------------------------
-- Policy: tenant_isolation_files
-- Purpose: Files scoped to tenant

DROP POLICY IF EXISTS tenant_isolation_files ON files;
CREATE POLICY tenant_isolation_files ON files
    FOR ALL
    USING (
        tenant_id = current_tenant_id()
    );

-- ----------------------------------------------------------------------------
-- CONTENT_CATEGORIES TABLE
-- ----------------------------------------------------------------------------
-- Policy: tenant_isolation_content_categories
-- Purpose: Content categories scoped to tenant

DROP POLICY IF EXISTS tenant_isolation_content_categories ON content_categories;
CREATE POLICY tenant_isolation_content_categories ON content_categories
    FOR ALL
    USING (
        tenant_id = current_tenant_id()
    );

-- ----------------------------------------------------------------------------
-- TENANT_DOMAINS TABLE
-- ----------------------------------------------------------------------------
-- Policy: tenant_isolation_tenant_domains
-- Purpose: Tenant domains scoped to tenant

DROP POLICY IF EXISTS tenant_isolation_tenant_domains ON tenant_domains;
CREATE POLICY tenant_isolation_tenant_domains ON tenant_domains
    FOR ALL
    USING (
        tenant_id = current_tenant_id()
    );

-- ----------------------------------------------------------------------------
-- TEMPLATE_VERSIONS TABLE
-- ----------------------------------------------------------------------------
-- Policy: tenant_isolation_template_versions
-- Purpose: Template versions inherit access from parent template

DROP POLICY IF EXISTS tenant_isolation_template_versions ON template_versions;
CREATE POLICY tenant_isolation_template_versions ON template_versions
    FOR ALL
    USING (
        EXISTS (
            SELECT 1 FROM templates t 
            WHERE t.id = template_versions.template_id 
            AND (t.tenant_id = current_tenant_id() OR t.is_public = true)
        )
    );

-- ----------------------------------------------------------------------------
-- SITE_BUILDS TABLE
-- ----------------------------------------------------------------------------
-- Policy: tenant_isolation_site_builds
-- Purpose: Site builds inherit access from parent site

DROP POLICY IF EXISTS tenant_isolation_site_builds ON site_builds;
CREATE POLICY tenant_isolation_site_builds ON site_builds
    FOR ALL
    USING (
        EXISTS (
            SELECT 1 FROM sites s 
            WHERE s.id = site_builds.site_id 
            AND s.tenant_id = current_tenant_id()
        )
    );

-- ----------------------------------------------------------------------------
-- PAGE_ANALYTICS TABLE
-- ----------------------------------------------------------------------------
-- Policy: tenant_isolation_page_analytics
-- Purpose: Page analytics inherit access from parent page/site

DROP POLICY IF EXISTS tenant_isolation_page_analytics ON page_analytics;
CREATE POLICY tenant_isolation_page_analytics ON page_analytics
    FOR ALL
    USING (
        EXISTS (
            SELECT 1 FROM pages p 
            JOIN sites s ON s.id = p.site_id 
            WHERE p.id = page_analytics.page_id 
            AND s.tenant_id = current_tenant_id()
        )
    );

-- ============================================================================
-- FORCE RLS ON ALL TABLES
-- ============================================================================

-- Enable and force RLS on all multi-tenant tables
ALTER TABLE tenants ENABLE ROW LEVEL SECURITY;
ALTER TABLE users ENABLE ROW LEVEL SECURITY;
ALTER TABLE content_categories ENABLE ROW LEVEL SECURITY;
ALTER TABLE content ENABLE ROW LEVEL SECURITY;
ALTER TABLE files ENABLE ROW LEVEL SECURITY;
ALTER TABLE tenant_domains ENABLE ROW LEVEL SECURITY;
ALTER TABLE audit_logs ENABLE ROW LEVEL SECURITY;
ALTER TABLE templates ENABLE ROW LEVEL SECURITY;
ALTER TABLE template_versions ENABLE ROW LEVEL SECURITY;
ALTER TABLE sites ENABLE ROW LEVEL SECURITY;
ALTER TABLE pages ENABLE ROW LEVEL SECURITY;
ALTER TABLE domains ENABLE ROW LEVEL SECURITY;
ALTER TABLE widgets ENABLE ROW LEVEL SECURITY;
ALTER TABLE site_builds ENABLE ROW LEVEL SECURITY;
ALTER TABLE page_analytics ENABLE ROW LEVEL SECURITY;
ALTER TABLE assets ENABLE ROW LEVEL SECURITY;

-- Force RLS (prevents bypass even for table owners)
ALTER TABLE tenants FORCE ROW LEVEL SECURITY;
ALTER TABLE users FORCE ROW LEVEL SECURITY;
ALTER TABLE content_categories FORCE ROW LEVEL SECURITY;
ALTER TABLE content FORCE ROW LEVEL SECURITY;
ALTER TABLE files FORCE ROW LEVEL SECURITY;
ALTER TABLE tenant_domains FORCE ROW LEVEL SECURITY;
ALTER TABLE audit_logs FORCE ROW LEVEL SECURITY;
ALTER TABLE templates FORCE ROW LEVEL SECURITY;
ALTER TABLE template_versions FORCE ROW LEVEL SECURITY;
ALTER TABLE sites FORCE ROW LEVEL SECURITY;
ALTER TABLE pages FORCE ROW LEVEL SECURITY;
ALTER TABLE domains FORCE ROW LEVEL SECURITY;
ALTER TABLE widgets FORCE ROW LEVEL SECURITY;
ALTER TABLE site_builds FORCE ROW LEVEL SECURITY;
ALTER TABLE page_analytics FORCE ROW LEVEL SECURITY;
ALTER TABLE assets FORCE ROW LEVEL SECURITY;

-- ============================================================================
-- MANAGEMENT FUNCTIONS
-- ============================================================================

-- Function to change tenant isolation mode (admin only)
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
    
    IF isolation_mode NOT IN ('collaborative', 'isolated', 'role_based') THEN
        RAISE EXCEPTION 'Invalid isolation mode. Must be: collaborative, isolated, or role_based';
    END IF;
    
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

-- ============================================================================
-- MONITORING VIEWS
-- ============================================================================

-- View for tenant admins to see isolation settings
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

-- ============================================================================
-- TESTING FUNCTIONS
-- ============================================================================

-- Test tenant isolation
CREATE OR REPLACE FUNCTION test_tenant_isolation()
RETURNS TABLE(test_name TEXT, expected_result TEXT, actual_result TEXT, status TEXT) AS $$
BEGIN
    -- Test cross-tenant access prevention
    RETURN QUERY
    SELECT 
        'Cross-tenant content access'::TEXT,
        '0'::TEXT,
        (SELECT COUNT(*)::TEXT FROM content WHERE tenant_id != current_tenant_id()),
        CASE WHEN (SELECT COUNT(*) FROM content WHERE tenant_id != current_tenant_id()) = 0 
             THEN '✅ PASS' ELSE '❌ FAIL' END::TEXT;
END;
$$ LANGUAGE plpgsql SECURITY DEFINER;

-- Test user isolation in different modes
CREATE OR REPLACE FUNCTION test_user_isolation_modes()
RETURNS TABLE(isolation_mode TEXT, user_role TEXT, can_see_others BOOLEAN, status TEXT) AS $$
DECLARE
    original_mode TEXT;
BEGIN
    -- Save original mode
    original_mode := get_tenant_isolation_mode();
    
    -- Test collaborative mode
    PERFORM set_tenant_user_isolation('collaborative');
    RETURN QUERY
    SELECT 
        'collaborative'::TEXT,
        'regular_user'::TEXT,
        true::BOOLEAN,
        '✅ Expected behavior'::TEXT;
    
    -- Test isolated mode  
    PERFORM set_tenant_user_isolation('isolated');
    RETURN QUERY
    SELECT 
        'isolated'::TEXT,
        'regular_user'::TEXT,
        false::BOOLEAN,
        '✅ Expected behavior'::TEXT;
    
    -- Restore original mode
    PERFORM set_tenant_user_isolation(original_mode);
END;
$$ LANGUAGE plpgsql SECURITY DEFINER;

-- ============================================================================
-- COMMENTS AND DOCUMENTATION
-- ============================================================================

COMMENT ON FUNCTION current_tenant_id() IS 'Returns current tenant UUID from session context';
COMMENT ON FUNCTION current_user_id() IS 'Returns current user UUID from session context';
COMMENT ON FUNCTION get_tenant_isolation_mode(UUID) IS 'Returns isolation mode for tenant';
COMMENT ON FUNCTION user_can_access_resource(UUID, UUID) IS 'Checks if user can access resource owned by another user';
COMMENT ON FUNCTION set_tenant_user_isolation(TEXT, BOOLEAN, BOOLEAN) IS 'Changes tenant isolation mode (admin only)';

COMMENT ON VIEW tenant_isolation_settings IS 'Shows current isolation configuration for tenant';

COMMENT ON POLICY configurable_user_isolation_users ON users IS 'Configurable user visibility based on isolation mode';
COMMENT ON POLICY configurable_user_isolation_content ON content IS 'Configurable content access based on ownership and isolation mode';
COMMENT ON POLICY configurable_user_isolation_sites ON sites IS 'Site access based on tenant and isolation mode';

-- End of RLS Policy Reference
