-- Migration: Create QuillSpace backup tables for Wix CMS collections
-- This enables bidirectional sync using externalId as the bridge

-- Authors backup table (mirrors Wix Authors collection exactly)
CREATE TABLE IF NOT EXISTS authors (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    
    -- Core author data (matches Wix Authors schema exactly)
    name TEXT NOT NULL, -- Primary field in Wix
    tagline TEXT,
    bio TEXT, -- Rich text field in Wix
    portrait_image TEXT, -- Image field in Wix (stores URL or Wix media ID)
    slug TEXT,
    external_id TEXT, -- External Id field for QuillSpace sync
    
    -- Social media fields (URL/Text fields in Wix)
    facebook TEXT,
    x TEXT, -- Twitter/X field
    instagram TEXT,
    email TEXT,
    phone TEXT,
    address TEXT,
    
    -- Sync metadata
    wix_id TEXT, -- The _id from Wix Authors collection
    wix_site_id TEXT NOT NULL,
    last_synced_at TIMESTAMPTZ,
    sync_status TEXT DEFAULT 'pending' CHECK (sync_status IN ('pending', 'synced', 'conflict', 'error')),
    
    -- Standard fields
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    
    -- Constraints
    UNIQUE(tenant_id, slug),
    UNIQUE(tenant_id, external_id), -- External Id should be unique per tenant
    UNIQUE(wix_site_id, wix_id) -- Ensure one-to-one mapping with Wix
);

-- Books backup table (mirrors Wix Books collection exactly)
CREATE TABLE IF NOT EXISTS books (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    author_id UUID REFERENCES authors(id) ON DELETE SET NULL,
    
    -- Core book data (matches Wix Books schema exactly)
    title TEXT NOT NULL, -- Primary field in Wix
    sub_title TEXT, -- "Sub Title" field in Wix
    description TEXT, -- Rich text field in Wix
    cover_image TEXT, -- Image field in Wix
    tilted_cover_image TEXT, -- "Tilted Cover Image" field in Wix
    buy_link TEXT, -- URL field in Wix
    status TEXT, -- Text field in Wix
    featured BOOLEAN DEFAULT FALSE, -- Boolean field in Wix
    price DECIMAL(10,2), -- Number field in Wix
    external_id TEXT, -- External Id field for QuillSpace sync
    
    -- Sync metadata
    wix_id TEXT, -- The _id from Wix Books collection
    wix_site_id TEXT NOT NULL,
    last_synced_at TIMESTAMPTZ,
    sync_status TEXT DEFAULT 'pending' CHECK (sync_status IN ('pending', 'synced', 'conflict', 'error')),
    
    -- Standard fields
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    
    -- Constraints
    UNIQUE(tenant_id, external_id), -- External Id should be unique per tenant
    UNIQUE(wix_site_id, wix_id) -- Ensure one-to-one mapping with Wix
);

-- Sync log table for tracking all sync operations
CREATE TABLE IF NOT EXISTS wix_sync_log (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    
    -- Sync operation details
    operation_type TEXT NOT NULL CHECK (operation_type IN ('create', 'update', 'delete', 'sync')),
    collection_name TEXT NOT NULL CHECK (collection_name IN ('authors', 'books')),
    record_id UUID NOT NULL, -- QuillSpace record ID
    wix_record_id TEXT, -- Wix _id
    wix_site_id TEXT NOT NULL,
    
    -- Operation results
    status TEXT NOT NULL CHECK (status IN ('success', 'error', 'conflict')),
    error_message TEXT,
    
    -- Data snapshots (for conflict resolution)
    quillspace_data JSONB,
    wix_data JSONB,
    
    -- Timestamps
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- RLS Policies for tenant isolation
ALTER TABLE authors ENABLE ROW LEVEL SECURITY;
ALTER TABLE books ENABLE ROW LEVEL SECURITY;
ALTER TABLE wix_sync_log ENABLE ROW LEVEL SECURITY;

-- Authors policies
CREATE POLICY authors_tenant_isolation ON authors
    FOR ALL USING (tenant_id = current_setting('app.current_tenant_id')::UUID);

-- Books policies  
CREATE POLICY books_tenant_isolation ON books
    FOR ALL USING (tenant_id = current_setting('app.current_tenant_id')::UUID);

-- Sync log policies
CREATE POLICY wix_sync_log_tenant_isolation ON wix_sync_log
    FOR ALL USING (tenant_id = current_setting('app.current_tenant_id')::UUID);

-- Indexes for performance
CREATE INDEX idx_authors_tenant_id ON authors(tenant_id);
CREATE INDEX idx_authors_wix_site_id ON authors(wix_site_id);
CREATE INDEX idx_authors_wix_id ON authors(wix_id);
CREATE INDEX idx_authors_external_id ON authors(tenant_id, external_id);
CREATE INDEX idx_authors_sync_status ON authors(sync_status);

CREATE INDEX idx_books_tenant_id ON books(tenant_id);
CREATE INDEX idx_books_author_id ON books(author_id);
CREATE INDEX idx_books_wix_site_id ON books(wix_site_id);
CREATE INDEX idx_books_wix_id ON books(wix_id);
CREATE INDEX idx_books_external_id ON books(tenant_id, external_id);
CREATE INDEX idx_books_sync_status ON books(sync_status);

CREATE INDEX idx_wix_sync_log_tenant_id ON wix_sync_log(tenant_id);
CREATE INDEX idx_wix_sync_log_record_id ON wix_sync_log(record_id);
CREATE INDEX idx_wix_sync_log_wix_record_id ON wix_sync_log(wix_record_id);
CREATE INDEX idx_wix_sync_log_created_at ON wix_sync_log(created_at);

-- Updated at triggers
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

CREATE TRIGGER update_authors_updated_at BEFORE UPDATE ON authors
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_books_updated_at BEFORE UPDATE ON books
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- Insert initial data for Yasin (example)
-- This would typically be done through the sync service
INSERT INTO authors (
    tenant_id, 
    name, 
    tagline, 
    bio,
    email,
    phone,
    address,
    x,
    facebook,
    instagram,
    wix_id,
    wix_site_id,
    sync_status,
    external_id
) VALUES (
    '22222222-2222-2222-2222-222222222222'::UUID, -- Yasin's tenant
    'Yasin Kakande',
    'Official Website of Best Selling Author',
    'A native of Uganda, Yasin Kakande has been a Middle East journalist for more than a decade...',
    'yasinkak@gmail.com',
    '+17815410081',
    '500 Terry Francine St. San Francisco, CA 94158',
    'https://x.com/yasikak',
    'https://www.facebook.com/yasikakande/',
    'https://www.instagram.com/yasikak/?hl=en',
    '31ac11d6-ecb6-4348-a3b4-125cb1965d39', -- Yasin's Wix _id
    '1e4e0091-f4d5-4a4c-a66a-4d09e7a5b4e9', -- Yasin's site
    'synced',
    '22222222-2222-2222-2222-222222222222' -- External ID = tenant ID for this example
) ON CONFLICT DO NOTHING;
