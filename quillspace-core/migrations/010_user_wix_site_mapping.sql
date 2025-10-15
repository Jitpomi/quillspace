-- User to Wix Site Mapping
-- Explicit relationship between QuillSpace users and their Wix sites built by the team

-- Simple table to map users to their Wix sites using Wix's native site IDs
CREATE TABLE user_wix_sites (
    -- Use Wix site ID as primary key since it's already unique
    wix_site_id TEXT PRIMARY KEY,       -- Site ID from Wix (e.g., '1e4a0091-f1d5-4a4c-a66e-4d09a75b4e9')
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    
    -- Optional overrides (pulled from Wix API if not set)
    display_name TEXT,                   -- Override site name if needed
    custom_domain TEXT,                  -- Custom domain if configured
    
    -- Project information
    project_status project_status_enum NOT NULL DEFAULT 'active',
    service_type service_type_enum NOT NULL DEFAULT 'build_and_manage',
    
    -- Permissions
    client_can_edit BOOLEAN NOT NULL DEFAULT TRUE,
    client_can_publish BOOLEAN NOT NULL DEFAULT TRUE,
    
    -- Metadata
    metadata JSONB DEFAULT '{}',
    
    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    -- Constraints
    UNIQUE(user_id, wix_site_id)  -- One user per site (but allows multiple sites per user)
);

-- Enums for project status and service type
CREATE TYPE project_status_enum AS ENUM (
    'development',    -- Being built by QuillSpace team
    'review',        -- Ready for client review
    'active',        -- Live and client can edit
    'maintenance',   -- Temporarily unavailable
    'completed',     -- Project finished
    'cancelled'      -- Project cancelled
);

CREATE TYPE service_type_enum AS ENUM (
    'build_and_manage',     -- Full service (option 2)
    'consultation_only',    -- Just consultation
    'maintenance_only',     -- Ongoing maintenance
    'migration',           -- Migrating existing site
    'redesign'             -- Redesigning existing site
);

-- Indexes for performance
CREATE INDEX idx_user_wix_sites_user ON user_wix_sites(user_id);
CREATE INDEX idx_user_wix_sites_tenant ON user_wix_sites(tenant_id);
CREATE INDEX idx_user_wix_sites_status ON user_wix_sites(project_status);

-- Row Level Security
ALTER TABLE user_wix_sites ENABLE ROW LEVEL SECURITY;

-- RLS Policies
CREATE POLICY user_wix_sites_tenant_isolation ON user_wix_sites
    FOR ALL USING (tenant_id = current_setting('app.current_tenant_id')::UUID);

CREATE POLICY user_wix_sites_user_access ON user_wix_sites
    FOR ALL USING (
        user_id = current_setting('app.current_user_id')::UUID 
        OR current_setting('app.user_role') = 'admin'
    );

-- Update trigger
CREATE TRIGGER update_user_wix_sites_updated_at 
    BEFORE UPDATE ON user_wix_sites 
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- Insert Yasin's website mapping (simple!)
-- Just map the Wix site ID to the user - everything else comes from Wix API
INSERT INTO user_wix_sites (
    wix_site_id,
    tenant_id,
    user_id,
    display_name,
    project_status,
    service_type,
    client_can_edit,
    client_can_publish,
    metadata
) 
SELECT 
    '1e4a0091-f1d5-4a4c-a66e-4d09a75b4e9' as wix_site_id,  -- From Wix dashboard URL
    t.id as tenant_id,
    u.id as user_id,
    'Yasin Kakande' as display_name,  -- Optional override
    'active' as project_status,
    'build_and_manage' as service_type,
    TRUE as client_can_edit,
    TRUE as client_can_publish,
    jsonb_build_object(
        'created_by', 'quillspace_team',
        'service_start_date', NOW(),
        'notes', 'Author website built by QuillSpace team'
    ) as metadata
FROM tenants t
CROSS JOIN users u
WHERE t.name = 'black-writers-ink' 
AND (u.email ILIKE '%yasin%' OR u.name ILIKE '%yasin%')
LIMIT 1
ON CONFLICT (wix_site_id) DO NOTHING;

-- Verify the mapping
SELECT 
    uws.wix_site_id,
    uws.display_name,
    uws.project_status,
    uws.service_type,
    u.name as user_name,
    u.email as user_email,
    t.name as tenant_name
FROM user_wix_sites uws
JOIN users u ON uws.user_id = u.id
JOIN tenants t ON uws.tenant_id = t.id
ORDER BY uws.created_at DESC;
