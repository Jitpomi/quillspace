-- Complete QuillSpace Database Setup
-- This single migration sets up the entire database with working authentication

-- Enable UUID extension
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Create user roles enum
CREATE TYPE user_role AS ENUM ('admin', 'editor', 'viewer');

-- Create tenants table
CREATE TABLE tenants (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(255) NOT NULL,
    slug VARCHAR(100) UNIQUE NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Create users table
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    email VARCHAR(255) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    first_name VARCHAR(255) NOT NULL,
    last_name VARCHAR(255) NOT NULL,
    role user_role NOT NULL DEFAULT 'viewer',
    active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Create content table
CREATE TABLE content (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    author_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    title VARCHAR(500) NOT NULL,
    content TEXT,
    status VARCHAR(50) DEFAULT 'draft',
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Create updated_at trigger function
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Add triggers for updated_at
CREATE TRIGGER update_tenants_updated_at BEFORE UPDATE ON tenants
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_users_updated_at BEFORE UPDATE ON users
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_content_updated_at BEFORE UPDATE ON content
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- Create authentication function that works without RLS conflicts
CREATE OR REPLACE FUNCTION authenticate_user(user_email TEXT)
RETURNS TABLE(
    id UUID,
    tenant_id UUID,
    email TEXT,
    first_name TEXT,
    last_name TEXT,
    role TEXT,
    active BOOLEAN,
    created_at TIMESTAMPTZ,
    updated_at TIMESTAMPTZ,
    password_hash TEXT
) 
SECURITY DEFINER
LANGUAGE plpgsql
AS $$
BEGIN
    RETURN QUERY
    SELECT 
        u.id,
        u.tenant_id,
        u.email::TEXT,
        u.first_name::TEXT,
        u.last_name::TEXT,
        u.role::TEXT,
        u.active,
        u.created_at,
        u.updated_at,
        u.password_hash::TEXT
    FROM users u
    WHERE u.email = user_email
    AND u.active = true;
END;
$$;

-- Insert QuillSpace System tenant (for admin and super user)
INSERT INTO tenants (id, name, slug, created_at, updated_at) 
VALUES (
    '11111111-1111-1111-1111-111111111111'::uuid,
    'QuillSpace System',
    'quillspace-system',
    NOW(),
    NOW()
);

-- Insert Black Writers Ink tenant
INSERT INTO tenants (id, name, slug, created_at, updated_at) 
VALUES (
    '22222222-2222-2222-2222-222222222222'::uuid,
    'Black Writers Ink',
    'black-writers-ink',
    NOW(),
    NOW()
);

-- Insert Greenlights tenant
INSERT INTO tenants (id, name, slug, created_at, updated_at) 
VALUES (
    '33333333-3333-3333-3333-333333333333'::uuid,
    'Greenlights',
    'greenlights',
    NOW(),
    NOW()
);

-- Insert system admin user (password: 'secret')
INSERT INTO users (id, email, password_hash, first_name, last_name, role, tenant_id, active, created_at, updated_at)
VALUES (
    'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa'::uuid,
    'admin@quillspace.com',
    '$2b$12$EixZaYVK1fsbw1ZfbX3OXePaWxn96p36WQoeG6Lruj3vjPGga31lW',
    'System',
    'Administrator', 
    'admin',
    '11111111-1111-1111-1111-111111111111'::uuid,
    true,
    NOW(),
    NOW()
);

-- Insert super user (can manage everything across all tenants) (password: 'secret')
INSERT INTO users (id, email, password_hash, first_name, last_name, role, tenant_id, active, created_at, updated_at)
VALUES (
    'cccccccc-cccc-cccc-cccc-cccccccccccc'::uuid,
    'super@quillspace.com',
    '$2b$12$EixZaYVK1fsbw1ZfbX3OXePaWxn96p36WQoeG6Lruj3vjPGga31lW',
    'Super',
    'User', 
    'admin',
    '11111111-1111-1111-1111-111111111111'::uuid,
    true,
    NOW(),
    NOW()
);

-- Insert Yasin Kakande in Black Writers Ink tenant (password: 'secret')
INSERT INTO users (id, email, password_hash, first_name, last_name, role, tenant_id, active, created_at, updated_at)
VALUES (
    'bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb'::uuid,
    'yasinkak@gmail.com',
    '$2b$12$EixZaYVK1fsbw1ZfbX3OXePaWxn96p36WQoeG6Lruj3vjPGga31lW',
    'Yasin',
    'Kakande', 
    'admin',
    '22222222-2222-2222-2222-222222222222'::uuid,
    true,
    NOW(),
    NOW()
);

-- Insert Josephine Nakigozi in Black Writers Ink tenant (password: 'secret')
INSERT INTO users (id, email, password_hash, first_name, last_name, role, tenant_id, active, created_at, updated_at)
VALUES (
    'dddddddd-dddd-dddd-dddd-dddddddddddd'::uuid,
    'joeykigozi@yahoo.co.uk',
    '$2b$12$EixZaYVK1fsbw1ZfbX3OXePaWxn96p36WQoeG6Lruj3vjPGga31lW',
    'Josephine',
    'Nakigozi', 
    'editor',
    '22222222-2222-2222-2222-222222222222'::uuid,
    true,
    NOW(),
    NOW()
);

-- Insert Matthew McConaughey in Greenlights tenant (password: 'secret')
INSERT INTO users (id, email, password_hash, first_name, last_name, role, tenant_id, active, created_at, updated_at)
VALUES (
    'eeeeeeee-eeee-eeee-eeee-eeeeeeeeeeee'::uuid,
    'matthew@greenlights.com',
    '$2b$12$EixZaYVK1fsbw1ZfbX3OXePaWxn96p36WQoeG6Lruj3vjPGga31lW',
    'Matthew',
    'McConaughey', 
    'admin',
    '33333333-3333-3333-3333-333333333333'::uuid,
    true,
    NOW(),
    NOW()
);

-- Enable RLS on tables
ALTER TABLE tenants ENABLE ROW LEVEL SECURITY;
ALTER TABLE users ENABLE ROW LEVEL SECURITY;
ALTER TABLE content ENABLE ROW LEVEL SECURITY;

-- Create simple RLS policies that work
CREATE POLICY tenant_isolation_tenants ON tenants
    FOR ALL
    USING (id = current_setting('app.current_tenant_id', true)::uuid);

CREATE POLICY tenant_isolation_users ON users
    FOR ALL 
    USING (tenant_id = current_setting('app.current_tenant_id', true)::uuid)
    WITH CHECK (tenant_id = current_setting('app.current_tenant_id', true)::uuid);

CREATE POLICY tenant_isolation_content ON content
    FOR ALL
    USING (tenant_id = current_setting('app.current_tenant_id', true)::uuid)
    WITH CHECK (tenant_id = current_setting('app.current_tenant_id', true)::uuid);

-- Make authenticate_user function owned by postgres so it bypasses RLS
ALTER FUNCTION authenticate_user(TEXT) OWNER TO postgres;
GRANT EXECUTE ON FUNCTION authenticate_user(TEXT) TO quillspace;

-- Create indexes for performance
CREATE INDEX idx_users_email ON users(email);
CREATE INDEX idx_users_tenant_id ON users(tenant_id);
CREATE INDEX idx_content_tenant_id ON content(tenant_id);
CREATE INDEX idx_content_author_id ON content(author_id);
