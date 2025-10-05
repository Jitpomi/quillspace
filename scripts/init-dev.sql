-- QuillSpace Development Database Initialization
-- This script sets up the development database with required extensions,
-- functions, and initial schema for multi-tenant architecture.

-- Enable required PostgreSQL extensions
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "pgcrypto";
CREATE EXTENSION IF NOT EXISTS "pg_stat_statements";

-- Create application roles
DO $$
BEGIN
    IF NOT EXISTS (SELECT FROM pg_catalog.pg_roles WHERE rolname = 'quillspace_app') THEN
        CREATE ROLE quillspace_app WITH LOGIN PASSWORD 'dev_password';
    END IF;
    
    IF NOT EXISTS (SELECT FROM pg_catalog.pg_roles WHERE rolname = 'quillspace_readonly') THEN
        CREATE ROLE quillspace_readonly WITH LOGIN PASSWORD 'dev_readonly_password';
    END IF;
END
$$;

-- Grant necessary permissions
GRANT CONNECT ON DATABASE quillspace_dev TO quillspace_app;
GRANT CONNECT ON DATABASE quillspace_dev TO quillspace_readonly;

-- Enable Row Level Security globally
ALTER DATABASE quillspace_dev SET row_security = on;

-- Create tenant context functions
CREATE OR REPLACE FUNCTION current_tenant_id()
RETURNS UUID AS $$
BEGIN
    RETURN current_setting('app.current_tenant_id', true)::UUID;
EXCEPTION
    WHEN OTHERS THEN
        RETURN NULL;
END;
$$ LANGUAGE plpgsql SECURITY DEFINER;

CREATE OR REPLACE FUNCTION set_current_tenant(tenant_uuid UUID)
RETURNS VOID AS $$
BEGIN
    PERFORM set_config('app.current_tenant_id', tenant_uuid::text, true);
END;
$$ LANGUAGE plpgsql SECURITY DEFINER;

-- Create audit trigger function
CREATE OR REPLACE FUNCTION audit_trigger_function()
RETURNS TRIGGER AS $$
BEGIN
    IF TG_OP = 'INSERT' THEN
        NEW.created_at = COALESCE(NEW.created_at, NOW());
        NEW.updated_at = COALESCE(NEW.updated_at, NOW());
        RETURN NEW;
    ELSIF TG_OP = 'UPDATE' THEN
        NEW.created_at = OLD.created_at; -- Preserve original created_at
        NEW.updated_at = NOW();
        RETURN NEW;
    END IF;
    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

-- Create tenants table
CREATE TABLE IF NOT EXISTS tenants (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(255) NOT NULL,
    slug VARCHAR(100) NOT NULL UNIQUE,
    domain VARCHAR(255) UNIQUE,
    status VARCHAR(20) NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'suspended', 'cancelled')),
    plan_type VARCHAR(20) NOT NULL DEFAULT 'starter' CHECK (plan_type IN ('starter', 'pro', 'enterprise')),
    settings JSONB DEFAULT '{}',
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Create indexes for tenants
CREATE INDEX IF NOT EXISTS idx_tenants_slug ON tenants(slug);
CREATE INDEX IF NOT EXISTS idx_tenants_domain ON tenants(domain) WHERE domain IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_tenants_status ON tenants(status);
CREATE INDEX IF NOT EXISTS idx_tenants_plan_type ON tenants(plan_type);

-- Create audit trigger for tenants
CREATE TRIGGER tenants_audit_trigger
    BEFORE INSERT OR UPDATE ON tenants
    FOR EACH ROW EXECUTE FUNCTION audit_trigger_function();

-- Create users table
CREATE TABLE IF NOT EXISTS users (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    email VARCHAR(255) NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    first_name VARCHAR(100),
    last_name VARCHAR(100),
    role VARCHAR(20) NOT NULL DEFAULT 'user' CHECK (role IN ('admin', 'editor', 'author', 'user')),
    active BOOLEAN DEFAULT true,
    email_verified BOOLEAN DEFAULT false,
    last_login TIMESTAMPTZ,
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    
    UNIQUE(tenant_id, email)
);

-- Enable RLS for users
ALTER TABLE users ENABLE ROW LEVEL SECURITY;

-- Create RLS policy for users
CREATE POLICY tenant_isolation_users ON users
    USING (tenant_id = current_tenant_id());

-- Create indexes for users
CREATE INDEX IF NOT EXISTS idx_users_tenant_id ON users(tenant_id);
CREATE INDEX IF NOT EXISTS idx_users_tenant_email ON users(tenant_id, email);
CREATE INDEX IF NOT EXISTS idx_users_tenant_role ON users(tenant_id, role);
CREATE INDEX IF NOT EXISTS idx_users_active ON users(active) WHERE active = true;

-- Create audit trigger for users
CREATE TRIGGER users_audit_trigger
    BEFORE INSERT OR UPDATE ON users
    FOR EACH ROW EXECUTE FUNCTION audit_trigger_function();

-- Create content_categories table
CREATE TABLE IF NOT EXISTS content_categories (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    slug VARCHAR(100) NOT NULL,
    description TEXT,
    parent_id UUID REFERENCES content_categories(id) ON DELETE SET NULL,
    sort_order INTEGER DEFAULT 0,
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    
    UNIQUE(tenant_id, slug)
);

-- Enable RLS for content_categories
ALTER TABLE content_categories ENABLE ROW LEVEL SECURITY;

-- Create RLS policy for content_categories
CREATE POLICY tenant_isolation_content_categories ON content_categories
    USING (tenant_id = current_tenant_id());

-- Create indexes for content_categories
CREATE INDEX IF NOT EXISTS idx_content_categories_tenant_id ON content_categories(tenant_id);
CREATE INDEX IF NOT EXISTS idx_content_categories_parent_id ON content_categories(parent_id);
CREATE INDEX IF NOT EXISTS idx_content_categories_sort_order ON content_categories(tenant_id, sort_order);

-- Create audit trigger for content_categories
CREATE TRIGGER content_categories_audit_trigger
    BEFORE INSERT OR UPDATE ON content_categories
    FOR EACH ROW EXECUTE FUNCTION audit_trigger_function();

-- Create content table
CREATE TABLE IF NOT EXISTS content (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    title VARCHAR(500) NOT NULL,
    slug VARCHAR(200) NOT NULL,
    content TEXT,
    excerpt TEXT,
    type VARCHAR(20) NOT NULL DEFAULT 'article' CHECK (type IN ('article', 'page', 'listing', 'product')),
    status VARCHAR(20) NOT NULL DEFAULT 'draft' CHECK (status IN ('draft', 'published', 'archived', 'deleted')),
    author_id UUID NOT NULL REFERENCES users(id),
    category_id UUID REFERENCES content_categories(id) ON DELETE SET NULL,
    featured_image_url TEXT,
    tags TEXT[] DEFAULT '{}',
    seo_title VARCHAR(255),
    seo_description TEXT,
    seo_keywords TEXT[],
    published_at TIMESTAMPTZ,
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    
    UNIQUE(tenant_id, slug)
);

-- Enable RLS for content
ALTER TABLE content ENABLE ROW LEVEL SECURITY;

-- Create RLS policy for content
CREATE POLICY tenant_isolation_content ON content
    USING (tenant_id = current_tenant_id());

-- Create indexes for content
CREATE INDEX IF NOT EXISTS idx_content_tenant_id ON content(tenant_id);
CREATE INDEX IF NOT EXISTS idx_content_tenant_status ON content(tenant_id, status);
CREATE INDEX IF NOT EXISTS idx_content_tenant_type ON content(tenant_id, type);
CREATE INDEX IF NOT EXISTS idx_content_tenant_published ON content(tenant_id, published_at DESC) WHERE status = 'published';
CREATE INDEX IF NOT EXISTS idx_content_author_id ON content(author_id);
CREATE INDEX IF NOT EXISTS idx_content_category_id ON content(category_id);
CREATE INDEX IF NOT EXISTS idx_content_tags ON content USING GIN(tags);

-- Create audit trigger for content
CREATE TRIGGER content_audit_trigger
    BEFORE INSERT OR UPDATE ON content
    FOR EACH ROW EXECUTE FUNCTION audit_trigger_function();

-- Create files table
CREATE TABLE IF NOT EXISTS files (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    filename VARCHAR(255) NOT NULL,
    original_name VARCHAR(255) NOT NULL,
    mime_type VARCHAR(100) NOT NULL,
    size_bytes BIGINT NOT NULL,
    storage_path TEXT NOT NULL,
    public_url TEXT,
    uploaded_by UUID NOT NULL REFERENCES users(id),
    folder VARCHAR(255) DEFAULT '',
    alt_text TEXT,
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Enable RLS for files
ALTER TABLE files ENABLE ROW LEVEL SECURITY;

-- Create RLS policy for files
CREATE POLICY tenant_isolation_files ON files
    USING (tenant_id = current_tenant_id());

-- Create indexes for files
CREATE INDEX IF NOT EXISTS idx_files_tenant_id ON files(tenant_id);
CREATE INDEX IF NOT EXISTS idx_files_tenant_folder ON files(tenant_id, folder);
CREATE INDEX IF NOT EXISTS idx_files_uploaded_by ON files(uploaded_by);
CREATE INDEX IF NOT EXISTS idx_files_mime_type ON files(mime_type);

-- Create audit trigger for files
CREATE TRIGGER files_audit_trigger
    BEFORE INSERT OR UPDATE ON files
    FOR EACH ROW EXECUTE FUNCTION audit_trigger_function();

-- Create tenant_domains table for custom domains
CREATE TABLE IF NOT EXISTS tenant_domains (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    domain VARCHAR(255) NOT NULL UNIQUE,
    verified BOOLEAN DEFAULT false,
    ssl_enabled BOOLEAN DEFAULT false,
    ssl_certificate TEXT,
    dns_configured BOOLEAN DEFAULT false,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Enable RLS for tenant_domains
ALTER TABLE tenant_domains ENABLE ROW LEVEL SECURITY;

-- Create RLS policy for tenant_domains
CREATE POLICY tenant_isolation_tenant_domains ON tenant_domains
    USING (tenant_id = current_tenant_id());

-- Create indexes for tenant_domains
CREATE INDEX IF NOT EXISTS idx_tenant_domains_tenant_id ON tenant_domains(tenant_id);
CREATE INDEX IF NOT EXISTS idx_tenant_domains_domain ON tenant_domains(domain);
CREATE INDEX IF NOT EXISTS idx_tenant_domains_verified ON tenant_domains(verified) WHERE verified = true;

-- Create audit trigger for tenant_domains
CREATE TRIGGER tenant_domains_audit_trigger
    BEFORE INSERT OR UPDATE ON tenant_domains
    FOR EACH ROW EXECUTE FUNCTION audit_trigger_function();

-- Create audit_logs table
CREATE TABLE IF NOT EXISTS audit_logs (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    user_id UUID REFERENCES users(id) ON DELETE SET NULL,
    action VARCHAR(100) NOT NULL,
    resource_type VARCHAR(50) NOT NULL,
    resource_id UUID,
    old_values JSONB,
    new_values JSONB,
    metadata JSONB DEFAULT '{}',
    ip_address INET,
    user_agent TEXT,
    timestamp TIMESTAMPTZ DEFAULT NOW()
);

-- Enable RLS for audit_logs
ALTER TABLE audit_logs ENABLE ROW LEVEL SECURITY;

-- Create RLS policy for audit_logs
CREATE POLICY tenant_isolation_audit_logs ON audit_logs
    USING (tenant_id = current_tenant_id());

-- Create indexes for audit_logs
CREATE INDEX IF NOT EXISTS idx_audit_logs_tenant_id ON audit_logs(tenant_id);
CREATE INDEX IF NOT EXISTS idx_audit_logs_user_id ON audit_logs(user_id);
CREATE INDEX IF NOT EXISTS idx_audit_logs_timestamp ON audit_logs(timestamp DESC);
CREATE INDEX IF NOT EXISTS idx_audit_logs_resource ON audit_logs(resource_type, resource_id);

-- Grant permissions to application roles
GRANT USAGE ON SCHEMA public TO quillspace_app, quillspace_readonly;
GRANT ALL PRIVILEGES ON ALL TABLES IN SCHEMA public TO quillspace_app;
GRANT SELECT ON ALL TABLES IN SCHEMA public TO quillspace_readonly;
GRANT ALL PRIVILEGES ON ALL SEQUENCES IN SCHEMA public TO quillspace_app;
GRANT EXECUTE ON ALL FUNCTIONS IN SCHEMA public TO quillspace_app, quillspace_readonly;

-- Set default privileges for future objects
ALTER DEFAULT PRIVILEGES IN SCHEMA public GRANT ALL ON TABLES TO quillspace_app;
ALTER DEFAULT PRIVILEGES IN SCHEMA public GRANT SELECT ON TABLES TO quillspace_readonly;
ALTER DEFAULT PRIVILEGES IN SCHEMA public GRANT ALL ON SEQUENCES TO quillspace_app;
ALTER DEFAULT PRIVILEGES IN SCHEMA public GRANT EXECUTE ON FUNCTIONS TO quillspace_app, quillspace_readonly;

-- Create development-specific views for easier querying
CREATE OR REPLACE VIEW tenant_stats AS
SELECT 
    t.id,
    t.name,
    t.slug,
    t.status,
    t.plan_type,
    COUNT(DISTINCT u.id) as user_count,
    COUNT(DISTINCT c.id) as content_count,
    COUNT(DISTINCT f.id) as file_count,
    SUM(f.size_bytes) as total_storage_bytes,
    t.created_at
FROM tenants t
LEFT JOIN users u ON t.id = u.tenant_id AND u.active = true
LEFT JOIN content c ON t.id = c.tenant_id AND c.status != 'deleted'
LEFT JOIN files f ON t.id = f.tenant_id
GROUP BY t.id, t.name, t.slug, t.status, t.plan_type, t.created_at
ORDER BY t.created_at DESC;

-- Grant access to the view
GRANT SELECT ON tenant_stats TO quillspace_app, quillspace_readonly;

-- Log successful initialization
INSERT INTO audit_logs (tenant_id, action, resource_type, metadata, timestamp)
VALUES (
    uuid_generate_v4(), -- Temporary tenant_id for system actions
    'database_initialized',
    'system',
    '{"version": "0.1.0", "environment": "development"}',
    NOW()
) ON CONFLICT DO NOTHING;

-- Display initialization summary
DO $$
BEGIN
    RAISE NOTICE 'QuillSpace development database initialized successfully!';
    RAISE NOTICE 'Extensions enabled: uuid-ossp, pgcrypto, pg_stat_statements';
    RAISE NOTICE 'Row Level Security enabled globally';
    RAISE NOTICE 'Tables created: tenants, users, content_categories, content, files, tenant_domains, audit_logs';
    RAISE NOTICE 'Roles created: quillspace_app, quillspace_readonly';
    RAISE NOTICE 'Ready for application connection!';
END
$$;
