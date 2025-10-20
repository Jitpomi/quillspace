-- Migration: 010_connected_websites.sql
-- Create connected_websites table for storing external website connections

-- Create enum for builder types
CREATE TYPE builder_type AS ENUM ('wix', 'squarespace', 'wordpress', 'custom');

-- Create enum for connection status
CREATE TYPE connection_status AS ENUM ('active', 'inactive', 'pending', 'syncing', 'error');

-- Create connected_websites table
CREATE TABLE connected_websites (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    builder_type builder_type NOT NULL,
    external_site_id VARCHAR(255) NOT NULL,
    name VARCHAR(255) NOT NULL,
    url TEXT,
    domain VARCHAR(255),
    status connection_status NOT NULL DEFAULT 'pending',
    last_sync TIMESTAMP WITH TIME ZONE,
    sync_error TEXT,
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    
    -- Constraints
    CONSTRAINT unique_site_per_user UNIQUE (user_id, builder_type, external_site_id)
);

-- Create indexes for performance
CREATE INDEX idx_connected_websites_tenant_id ON connected_websites(tenant_id);
CREATE INDEX idx_connected_websites_user_id ON connected_websites(user_id);
CREATE INDEX idx_connected_websites_builder_type ON connected_websites(builder_type);
CREATE INDEX idx_connected_websites_status ON connected_websites(status);
CREATE INDEX idx_connected_websites_external_site_id ON connected_websites(external_site_id);

-- Create updated_at trigger
CREATE OR REPLACE FUNCTION update_connected_websites_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_connected_websites_updated_at
    BEFORE UPDATE ON connected_websites
    FOR EACH ROW
    EXECUTE FUNCTION update_connected_websites_updated_at();

-- Enable Row Level Security
ALTER TABLE connected_websites ENABLE ROW LEVEL SECURITY;

-- Create RLS policies
CREATE POLICY connected_websites_tenant_isolation ON connected_websites
    FOR ALL
    TO authenticated
    USING (tenant_id = current_setting('app.current_tenant_id')::UUID);

CREATE POLICY connected_websites_user_access ON connected_websites
    FOR ALL
    TO authenticated
    USING (
        user_id = current_setting('app.current_user_id')::UUID
        OR 
        -- Allow tenant admins to see all websites in their tenant
        EXISTS (
            SELECT 1 FROM users 
            WHERE id = current_setting('app.current_user_id')::UUID 
            AND tenant_id = connected_websites.tenant_id
            AND role = 'admin'
        )
    );

-- Grant permissions
GRANT ALL ON connected_websites TO quillspace;
GRANT USAGE ON TYPE builder_type TO quillspace;
GRANT USAGE ON TYPE connection_status TO quillspace;

-- Add comments
COMMENT ON TABLE connected_websites IS 'Stores external website connections for users';
COMMENT ON COLUMN connected_websites.builder_type IS 'Type of website builder (wix, squarespace, wordpress, custom)';
COMMENT ON COLUMN connected_websites.external_site_id IS 'External site ID from the builder platform';
COMMENT ON COLUMN connected_websites.status IS 'Current connection status';
COMMENT ON COLUMN connected_websites.metadata IS 'Additional metadata specific to the builder type';
