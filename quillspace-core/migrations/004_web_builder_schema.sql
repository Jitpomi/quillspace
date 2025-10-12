-- QuillSpace Web Builder Database Schema
-- Phase 1: Multi-tenant web builder with Row-Level Security

-- Enable UUID extension if not already enabled
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Templates table with versioning support
CREATE TABLE templates (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id UUID REFERENCES tenants(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    category VARCHAR(100) DEFAULT 'custom',
    html_source TEXT NOT NULL, -- MiniJinja template source
    default_schema JSONB NOT NULL DEFAULT '{}', -- Widget configuration schema
    preview_image_url TEXT,
    is_public BOOLEAN DEFAULT false,
    version INTEGER DEFAULT 1,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(tenant_id, name, version)
);

-- Template version history
CREATE TABLE template_versions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    template_id UUID REFERENCES templates(id) ON DELETE CASCADE,
    version INTEGER NOT NULL,
    html_source TEXT NOT NULL,
    schema JSONB NOT NULL DEFAULT '{}',
    changelog TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(template_id, version)
);

-- Author (website-builder)/sites
CREATE TABLE sites (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id UUID REFERENCES tenants(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    template_id UUID REFERENCES templates(id),
    custom_domain VARCHAR(255),
    subdomain VARCHAR(100) UNIQUE, -- author.quillspace.com
    is_published BOOLEAN DEFAULT false,
    seo_settings JSONB DEFAULT '{}',
    build_status VARCHAR(50) DEFAULT 'draft', -- draft, building, published, error
    theme_config JSONB DEFAULT '{}', -- Custom theme overrides
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    CONSTRAINT valid_subdomain CHECK (subdomain ~ '^[a-z0-9][a-z0-9-]*[a-z0-9]$')
);

-- Individual pages with Puck composition
CREATE TABLE pages (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    site_id UUID REFERENCES sites(id) ON DELETE CASCADE,
    slug VARCHAR(200) NOT NULL,
    title VARCHAR(500) NOT NULL,
    meta_description TEXT,
    meta_keywords TEXT,
    puck_data JSONB NOT NULL DEFAULT '{}', -- Puck JSON composition
    is_published BOOLEAN DEFAULT false,
    published_html TEXT, -- Pre-rendered HTML for SEO
    published_at TIMESTAMPTZ,
    sort_order INTEGER DEFAULT 0,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(site_id, slug),
    CONSTRAINT valid_slug CHECK (slug ~ '^[a-z0-9][a-z0-9-]*[a-z0-9]$|^[a-z0-9]$')
);

-- Custom domain management and verification
CREATE TABLE domains (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    site_id UUID REFERENCES sites(id) ON DELETE CASCADE,
    domain VARCHAR(255) NOT NULL UNIQUE,
    verification_token VARCHAR(255) NOT NULL,
    verification_method VARCHAR(50) DEFAULT 'txt', -- txt, cname, file
    is_verified BOOLEAN DEFAULT false,
    dns_configured BOOLEAN DEFAULT false,
    ssl_status VARCHAR(50) DEFAULT 'pending', -- pending, issued, error, expired
    ssl_expires_at TIMESTAMPTZ,
    last_checked_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    verified_at TIMESTAMPTZ,
    CONSTRAINT valid_domain CHECK (domain ~ '^[a-zA-Z0-9][a-zA-Z0-9-]*[a-zA-Z0-9]*\.[a-zA-Z]{2,}$')
);

-- Widget marketplace and components
CREATE TABLE widgets (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(255) NOT NULL,
    display_name VARCHAR(255) NOT NULL,
    description TEXT,
    category VARCHAR(100) DEFAULT 'custom',
    component_config JSONB NOT NULL, -- Puck component definition
    external_api_config JSONB DEFAULT '{}', -- API integration settings
    is_public BOOLEAN DEFAULT true,
    is_approved BOOLEAN DEFAULT false,
    version VARCHAR(20) DEFAULT '1.0.0',
    icon_url TEXT,
    documentation_url TEXT,
    created_by UUID REFERENCES users(id),
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(name, version)
);

-- Site build and deployment tracking
CREATE TABLE site_builds (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    site_id UUID REFERENCES sites(id) ON DELETE CASCADE,
    build_type VARCHAR(50) NOT NULL, -- ssg, ssr, preview
    status VARCHAR(50) DEFAULT 'pending', -- pending, building, success, error
    build_log TEXT,
    assets_url TEXT, -- Object storage URL
    error_message TEXT,
    build_duration_ms INTEGER,
    started_at TIMESTAMPTZ DEFAULT NOW(),
    completed_at TIMESTAMPTZ
);

-- Page analytics and performance metrics
CREATE TABLE page_analytics (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    page_id UUID REFERENCES pages(id) ON DELETE CASCADE,
    metric_type VARCHAR(50) NOT NULL, -- page_views, bounce_rate, load_time
    metric_value DECIMAL(10,4) NOT NULL,
    recorded_at TIMESTAMPTZ DEFAULT NOW(),
    user_agent TEXT,
    ip_address INET,
    referrer TEXT
);

-- Asset management for uploads
CREATE TABLE assets (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id UUID REFERENCES tenants(id) ON DELETE CASCADE,
    site_id UUID REFERENCES sites(id) ON DELETE CASCADE,
    filename VARCHAR(500) NOT NULL,
    original_filename VARCHAR(500) NOT NULL,
    mime_type VARCHAR(100) NOT NULL,
    file_size BIGINT NOT NULL,
    storage_path TEXT NOT NULL,
    cdn_url TEXT,
    alt_text TEXT,
    is_optimized BOOLEAN DEFAULT false,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Indexes for performance
CREATE INDEX idx_templates_tenant_id ON templates(tenant_id);
CREATE INDEX idx_templates_category ON templates(category);
CREATE INDEX idx_templates_public ON templates(is_public) WHERE is_public = true;

CREATE INDEX idx_sites_tenant_id ON sites(tenant_id);
CREATE INDEX idx_sites_subdomain ON sites(subdomain);
CREATE INDEX idx_sites_custom_domain ON sites(custom_domain) WHERE custom_domain IS NOT NULL;
CREATE INDEX idx_sites_published ON sites(is_published) WHERE is_published = true;

CREATE INDEX idx_pages_site_id ON pages(site_id);
CREATE INDEX idx_pages_slug ON pages(site_id, slug);
CREATE INDEX idx_pages_published ON pages(is_published) WHERE is_published = true;

CREATE INDEX idx_domains_domain ON domains(domain);
CREATE INDEX idx_domains_verified ON domains(is_verified) WHERE is_verified = true;

CREATE INDEX idx_widgets_category ON widgets(category);
CREATE INDEX idx_widgets_public ON widgets(is_public) WHERE is_public = true;

CREATE INDEX idx_site_builds_site_id ON site_builds(site_id);
CREATE INDEX idx_site_builds_status ON site_builds(status);

CREATE INDEX idx_assets_tenant_id ON assets(tenant_id);
CREATE INDEX idx_assets_site_id ON assets(site_id);

-- Enable Row-Level Security on all multi-tenant tables
ALTER TABLE templates ENABLE ROW LEVEL SECURITY;
ALTER TABLE template_versions ENABLE ROW LEVEL SECURITY;
ALTER TABLE sites ENABLE ROW LEVEL SECURITY;
ALTER TABLE pages ENABLE ROW LEVEL SECURITY;
ALTER TABLE domains ENABLE ROW LEVEL SECURITY;
ALTER TABLE site_builds ENABLE ROW LEVEL SECURITY;
ALTER TABLE page_analytics ENABLE ROW LEVEL SECURITY;
ALTER TABLE assets ENABLE ROW LEVEL SECURITY;

-- RLS Policies for tenant isolation
CREATE POLICY tenant_isolation_templates ON templates
    FOR ALL
    USING (tenant_id = current_setting('rls.tenant_id')::uuid)
    WITH CHECK (tenant_id = current_setting('rls.tenant_id')::uuid);

CREATE POLICY tenant_isolation_template_versions ON template_versions
    FOR ALL
    USING (EXISTS (
        SELECT 1 FROM templates t 
        WHERE t.id = template_versions.template_id 
        AND t.tenant_id = current_setting('rls.tenant_id')::uuid
    ));

CREATE POLICY tenant_isolation_sites ON sites
    FOR ALL
    USING (tenant_id = current_setting('rls.tenant_id')::uuid)
    WITH CHECK (tenant_id = current_setting('rls.tenant_id')::uuid);

CREATE POLICY tenant_isolation_pages ON pages
    FOR ALL
    USING (EXISTS (
        SELECT 1 FROM sites s 
        WHERE s.id = pages.site_id 
        AND s.tenant_id = current_setting('rls.tenant_id')::uuid
    ));

CREATE POLICY tenant_isolation_domains ON domains
    FOR ALL
    USING (EXISTS (
        SELECT 1 FROM sites s 
        WHERE s.id = domains.site_id 
        AND s.tenant_id = current_setting('rls.tenant_id')::uuid
    ));

CREATE POLICY tenant_isolation_site_builds ON site_builds
    FOR ALL
    USING (EXISTS (
        SELECT 1 FROM sites s 
        WHERE s.id = site_builds.site_id 
        AND s.tenant_id = current_setting('rls.tenant_id')::uuid
    ));

CREATE POLICY tenant_isolation_page_analytics ON page_analytics
    FOR ALL
    USING (EXISTS (
        SELECT 1 FROM pages p 
        JOIN sites s ON s.id = p.site_id 
        WHERE p.id = page_analytics.page_id 
        AND s.tenant_id = current_setting('rls.tenant_id')::uuid
    ));

CREATE POLICY tenant_isolation_assets ON assets
    FOR ALL
    USING (tenant_id = current_setting('rls.tenant_id')::uuid)
    WITH CHECK (tenant_id = current_setting('rls.tenant_id')::uuid);

-- Public widgets policy (can be viewed by all tenants)
CREATE POLICY public_widgets_read ON widgets
    FOR SELECT
    USING (is_public = true AND is_approved = true);

-- Widget owners can manage their own widgets
CREATE POLICY widget_owner_policy ON widgets
    FOR ALL
    USING (created_by = current_setting('rls.user_id')::uuid)
    WITH CHECK (created_by = current_setting('rls.user_id')::uuid);

-- Functions to auto-populate tenant_id and user_id on INSERT
CREATE OR REPLACE FUNCTION set_tenant_id()
RETURNS TRIGGER AS $$
BEGIN
    -- Only set tenant_id if it's not already provided and RLS setting exists
    IF NEW.tenant_id IS NULL AND current_setting('rls.tenant_id', true) IS NOT NULL THEN
        NEW.tenant_id = current_setting('rls.tenant_id')::uuid;
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION set_user_id()
RETURNS TRIGGER AS $$
BEGIN
    -- Only set created_by if it's not already provided and RLS setting exists
    IF NEW.created_by IS NULL AND current_setting('rls.user_id', true) IS NOT NULL THEN
        NEW.created_by = current_setting('rls.user_id')::uuid;
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Triggers to auto-populate tenant_id
CREATE TRIGGER set_tenant_id_templates
    BEFORE INSERT ON templates
    FOR EACH ROW EXECUTE FUNCTION set_tenant_id();

CREATE TRIGGER set_tenant_id_sites
    BEFORE INSERT ON sites
    FOR EACH ROW EXECUTE FUNCTION set_tenant_id();

CREATE TRIGGER set_tenant_id_assets
    BEFORE INSERT ON assets
    FOR EACH ROW EXECUTE FUNCTION set_tenant_id();

-- Triggers to auto-populate user_id
CREATE TRIGGER set_user_id_widgets
    BEFORE INSERT ON widgets
    FOR EACH ROW EXECUTE FUNCTION set_user_id();

-- Function to update updated_at timestamp
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Triggers for updated_at
CREATE TRIGGER update_templates_updated_at
    BEFORE UPDATE ON templates
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_sites_updated_at
    BEFORE UPDATE ON sites
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_pages_updated_at
    BEFORE UPDATE ON pages
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_widgets_updated_at
    BEFORE UPDATE ON widgets
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_assets_updated_at
    BEFORE UPDATE ON assets
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- Function to generate unique subdomain
CREATE OR REPLACE FUNCTION generate_unique_subdomain(base_name TEXT)
RETURNS TEXT AS $$
DECLARE
    clean_name TEXT;
    candidate TEXT;
    counter INTEGER := 0;
BEGIN
    -- Clean the base name: lowercase, replace spaces/special chars with hyphens
    clean_name := lower(regexp_replace(base_name, '[^a-zA-Z0-9]+', '-', 'g'));
    clean_name := trim(both '-' from clean_name);
    
    -- Ensure it starts and ends with alphanumeric
    clean_name := regexp_replace(clean_name, '^[^a-z0-9]+|[^a-z0-9]+$', '', 'g');
    
    -- Limit length
    clean_name := left(clean_name, 50);
    
    -- Try the base name first
    candidate := clean_name;
    
    -- Keep trying with incrementing numbers until we find a unique one
    WHILE EXISTS (SELECT 1 FROM sites WHERE subdomain = candidate) LOOP
        counter := counter + 1;
        candidate := clean_name || '-' || counter;
    END LOOP;
    
    RETURN candidate;
END;
$$ LANGUAGE plpgsql;

-- Function to validate and clean slugs
CREATE OR REPLACE FUNCTION clean_slug(input_slug TEXT)
RETURNS TEXT AS $$
DECLARE
    clean_slug TEXT;
BEGIN
    -- Convert to lowercase and replace spaces/special chars with hyphens
    clean_slug := lower(regexp_replace(input_slug, '[^a-zA-Z0-9]+', '-', 'g'));
    clean_slug := trim(both '-' from clean_slug);
    
    -- Ensure it starts and ends with alphanumeric
    clean_slug := regexp_replace(clean_slug, '^[^a-z0-9]+|[^a-z0-9]+$', '', 'g');
    
    -- Limit length
    clean_slug := left(clean_slug, 100);
    
    -- Default to 'page' if empty
    IF clean_slug = '' THEN
        clean_slug := 'page';
    END IF;
    
    RETURN clean_slug;
END;
$$ LANGUAGE plpgsql;

-- Insert default templates for new tenants
INSERT INTO templates (tenant_id, name, description, category, html_source, default_schema, is_public) VALUES
(
    '00000000-0000-0000-0000-000000000000', -- System tenant
    'minimal-blog',
    'A clean, minimal blog template perfect for writers and authors',
    'blog',
    '<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{{ page.title }} - {{ site.name }}</title>
    <meta name="description" content="{{ page.meta_description }}">
    <style>
        body { font-family: Georgia, serif; line-height: 1.6; max-width: 800px; margin: 0 auto; padding: 20px; }
        header { border-bottom: 1px solid #eee; margin-bottom: 2rem; padding-bottom: 1rem; }
        .site-title { font-size: 2rem; margin: 0; }
        .site-description { color: #666; margin: 0.5rem 0 0 0; }
        main { margin: 2rem 0; }
        .page-title { font-size: 2.5rem; margin-bottom: 1rem; }
        .page-content { font-size: 1.1rem; }
        footer { border-top: 1px solid #eee; margin-top: 3rem; padding-top: 1rem; text-align: center; color: #666; }
    </style>
</head>
<body>
    <header>
        <h1 class="site-title">{{ site.name }}</h1>
        <p class="site-description">{{ site.description }}</p>
    </header>
    <main>
        <h1 class="page-title">{{ page.title }}</h1>
        <div class="page-content">
            {{ puck_content | safe }}
        </div>
    </main>
    <footer>
        <p>&copy; {{ "now" | date(format="%Y") }} {{ site.name }}. Built with QuillSpace.</p>
    </footer>
</body>
</html>',
    '{
        "components": ["Hero", "TextBlock", "ImageBlock", "BlogPosts"],
        "layout": "single-column",
        "theme": {
            "primaryColor": "#333",
            "backgroundColor": "#fff",
            "fontFamily": "Georgia, serif"
        }
    }',
    true
),
(
    '00000000-0000-0000-0000-000000000000', -- System tenant
    'modern-portfolio',
    'A modern, responsive portfolio template for showcasing creative work',
    'portfolio',
    '<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{{ page.title }} - {{ site.name }}</title>
    <meta name="description" content="{{ page.meta_description }}">
    <style>
        * { margin: 0; padding: 0; box-sizing: border-box; }
        body { font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif; line-height: 1.6; }
        .container { max-width: 1200px; margin: 0 auto; padding: 0 20px; }
        header { background: #fff; box-shadow: 0 2px 10px rgba(0,0,0,0.1); position: fixed; width: 100%; top: 0; z-index: 1000; }
        nav { display: flex; justify-content: space-between; align-items: center; padding: 1rem 0; }
        .logo { font-size: 1.5rem; font-weight: bold; }
        main { margin-top: 80px; min-height: calc(100vh - 160px); }
        .hero { background: linear-gradient(135deg, #667eea 0%, #764ba2 100%); color: white; padding: 4rem 0; text-align: center; }
        .hero h1 { font-size: 3rem; margin-bottom: 1rem; }
        .content { padding: 3rem 0; }
        footer { background: #333; color: white; text-align: center; padding: 2rem 0; }
    </style>
</head>
<body>
    <header>
        <nav class="container">
            <div class="logo">{{ site.name }}</div>
        </nav>
    </header>
    <main>
        {% if page.slug == "home" %}
        <section class="hero">
            <div class="container">
                <h1>{{ page.title }}</h1>
                <p>{{ site.description }}</p>
            </div>
        </section>
        {% endif %}
        <section class="content">
            <div class="container">
                {{ puck_content | safe }}
            </div>
        </section>
    </main>
    <footer>
        <div class="container">
            <p>&copy; {{ "now" | date(format="%Y") }} {{ site.name }}. Built with QuillSpace.</p>
        </div>
    </footer>
</body>
</html>',
    '{
        "components": ["Hero", "Portfolio", "About", "Contact", "ImageGallery"],
        "layout": "full-width",
        "theme": {
            "primaryColor": "#667eea",
            "secondaryColor": "#764ba2",
            "backgroundColor": "#fff",
            "fontFamily": "system-ui"
        }
    }',
    true
);

-- Insert default widgets
INSERT INTO widgets (name, display_name, description, category, component_config, is_public, is_approved, created_by) VALUES
(
    'hero-section',
    'Hero Section',
    'A prominent hero section with title, subtitle, and call-to-action button',
    'layout',
    '{
        "fields": {
            "title": {"type": "text", "label": "Title"},
            "subtitle": {"type": "textarea", "label": "Subtitle"},
            "buttonText": {"type": "text", "label": "Button Text"},
            "buttonUrl": {"type": "text", "label": "Button URL"},
            "backgroundImage": {"type": "external", "label": "Background Image"}
        },
        "defaultProps": {
            "title": "Welcome to My Site",
            "subtitle": "This is a beautiful hero section",
            "buttonText": "Learn More",
            "buttonUrl": "#"
        }
    }',
    true,
    true,
    'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa' -- System admin
),
(
    'text-block',
    'Text Block',
    'Rich text content block with formatting options',
    'content',
    '{
        "fields": {
            "content": {"type": "textarea", "label": "Content"},
            "alignment": {"type": "select", "label": "Alignment", "options": [
                {"label": "Left", "value": "left"},
                {"label": "Center", "value": "center"},
                {"label": "Right", "value": "right"}
            ]}
        },
        "defaultProps": {
            "content": "Add your text content here...",
            "alignment": "left"
        }
    }',
    true,
    true,
    'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa' -- System admin
),
(
    'image-block',
    'Image Block',
    'Responsive image block with caption and alt text',
    'media',
    '{
        "fields": {
            "src": {"type": "external", "label": "Image URL"},
            "alt": {"type": "text", "label": "Alt Text"},
            "caption": {"type": "text", "label": "Caption"},
            "width": {"type": "select", "label": "Width", "options": [
                {"label": "Full Width", "value": "full"},
                {"label": "Large", "value": "large"},
                {"label": "Medium", "value": "medium"},
                {"label": "Small", "value": "small"}
            ]}
        },
        "defaultProps": {
            "alt": "Image description",
            "width": "full"
        }
    }',
    true,
    true,
    'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa' -- System admin
);

COMMENT ON TABLE templates IS 'MiniJinja templates for site rendering with versioning support';
COMMENT ON TABLE sites IS 'Author (website-builder) with custom domains and publishing settings';
COMMENT ON TABLE pages IS 'Individual pages with Puck composition data and SEO metadata';
COMMENT ON TABLE domains IS 'Custom domain management with DNS verification and SSL status';
COMMENT ON TABLE widgets IS 'Reusable components for the visual editor marketplace';
COMMENT ON TABLE site_builds IS 'Build and deployment tracking for static site generation';
COMMENT ON TABLE assets IS 'File uploads and media management with CDN integration';
