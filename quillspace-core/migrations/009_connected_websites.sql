-- Connected Websites System
-- Manages connections to external website builders (Wix, WordPress, etc.)

-- Website builder types
CREATE TYPE builder_type AS ENUM (
    'wix',
    'wordpress', 
    'squarespace',
    'jflux'
);

-- Website connection status
CREATE TYPE connection_status AS ENUM (
    'active',
    'inactive',
    'syncing',
    'error'
);

-- Website builder credentials (encrypted)
CREATE TABLE website_builder_credentials (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    builder_type builder_type NOT NULL,
    
    -- Encrypted credentials (JSON)
    encrypted_credentials TEXT NOT NULL,
    
    -- Connection metadata
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    last_sync TIMESTAMPTZ,
    sync_error TEXT,
    
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    -- Unique constraint per user per builder
    UNIQUE(user_id, builder_type)
);

-- Connected websites
CREATE TABLE connected_websites (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    credentials_id UUID NOT NULL REFERENCES website_builder_credentials(id) ON DELETE CASCADE,
    
    -- Website details
    builder_type builder_type NOT NULL,
    external_site_id TEXT NOT NULL, -- Site ID from external platform
    name TEXT NOT NULL,
    url TEXT,
    domain TEXT,
    
    -- Status and sync info
    status connection_status NOT NULL DEFAULT 'active',
    last_sync TIMESTAMPTZ,
    sync_error TEXT,
    
    -- Metadata from external platform
    metadata JSONB DEFAULT '{}',
    
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    -- Unique constraint per user per external site
    UNIQUE(user_id, builder_type, external_site_id)
);

-- Indexes for performance
CREATE INDEX idx_website_builder_credentials_user ON website_builder_credentials(user_id);
CREATE INDEX idx_website_builder_credentials_tenant ON website_builder_credentials(tenant_id);
CREATE INDEX idx_website_builder_credentials_builder ON website_builder_credentials(builder_type);

CREATE INDEX idx_connected_websites_user ON connected_websites(user_id);
CREATE INDEX idx_connected_websites_tenant ON connected_websites(tenant_id);
CREATE INDEX idx_connected_websites_builder ON connected_websites(builder_type);
CREATE INDEX idx_connected_websites_status ON connected_websites(status);
CREATE INDEX idx_connected_websites_credentials ON connected_websites(credentials_id);

-- Row Level Security
ALTER TABLE website_builder_credentials ENABLE ROW LEVEL SECURITY;
ALTER TABLE connected_websites ENABLE ROW LEVEL SECURITY;

-- RLS Policies for website_builder_credentials
CREATE POLICY website_builder_credentials_tenant_isolation ON website_builder_credentials
    FOR ALL USING (tenant_id = current_setting('app.current_tenant_id')::UUID);

CREATE POLICY website_builder_credentials_user_access ON website_builder_credentials
    FOR ALL USING (
        user_id = current_setting('app.current_user_id')::UUID 
        OR current_setting('app.user_role') = 'admin'
    );

-- RLS Policies for connected_websites
CREATE POLICY connected_websites_tenant_isolation ON connected_websites
    FOR ALL USING (tenant_id = current_setting('app.current_tenant_id')::UUID);

CREATE POLICY connected_websites_user_access ON connected_websites
    FOR ALL USING (
        user_id = current_setting('app.current_user_id')::UUID 
        OR current_setting('app.user_role') = 'admin'
    );

-- Update triggers
CREATE TRIGGER update_website_builder_credentials_updated_at 
    BEFORE UPDATE ON website_builder_credentials 
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_connected_websites_updated_at 
    BEFORE UPDATE ON connected_websites 
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- Insert Yasin's existing Wix website
-- Note: This would be done via API in production, but for demo purposes:
INSERT INTO website_builder_credentials (
    tenant_id, 
    user_id, 
    builder_type, 
    encrypted_credentials,
    is_active,
    last_sync
) VALUES (
    -- You'll need to replace these with actual UUIDs from your database
    (SELECT id FROM tenants WHERE name = 'black-writers-ink' LIMIT 1),
    (SELECT id FROM users WHERE email = 'yasin@example.com' LIMIT 1), -- Replace with Yasin's actual email
    'wix',
    'encrypted_wix_credentials_here', -- This would be encrypted in production
    TRUE,
    NOW()
) ON CONFLICT (user_id, builder_type) DO NOTHING;

-- Insert the connected website
INSERT INTO connected_websites (
    tenant_id,
    user_id, 
    credentials_id,
    builder_type,
    external_site_id,
    name,
    url,
    domain,
    status,
    last_sync,
    metadata
) VALUES (
    (SELECT id FROM tenants WHERE name = 'black-writers-ink' LIMIT 1),
    (SELECT id FROM users WHERE email = 'yasin@example.com' LIMIT 1), -- Replace with Yasin's actual email
    (SELECT id FROM website_builder_credentials WHERE user_id = (SELECT id FROM users WHERE email = 'yasin@example.com' LIMIT 1) AND builder_type = 'wix' LIMIT 1),
    'wix',
    '1e4a0091-f1d5-4a4c-a66e-4d09a75b4e9', -- From the Wix URL in the screenshot
    'Yasin Kakande',
    'https://yasinkakande.wixsite.com/yasin-kakande', -- From the screenshot
    'yasinkakande.wixsite.com',
    'active',
    NOW(),
    jsonb_build_object(
        'wix_site_id', '1e4a0091-f1d5-4a4c-a66e-4d09a75b4e9',
        'plan_type', 'free',
        'template_id', 'unknown',
        'last_published', NOW()
    )
) ON CONFLICT (user_id, builder_type, external_site_id) DO NOTHING;
