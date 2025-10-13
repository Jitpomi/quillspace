-- Migration: Puck + MiniJinja Architecture
-- Separates composition JSON (Puck) from template references
-- Enables template switching without content rewrite

-- Templates are versioned products
CREATE TABLE IF NOT EXISTS templates (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NULL,                    -- NULL = global/public template
    name TEXT NOT NULL,                     -- e.g. "literary-classic"
    version INT NOT NULL DEFAULT 1,
    display_name TEXT NOT NULL,             -- e.g. "Literary Classic"
    description TEXT,
    main_name TEXT NOT NULL DEFAULT 'author.html',  -- main template file
    html_main TEXT NOT NULL,                -- MiniJinja main template
    html_partials JSONB NOT NULL DEFAULT '{}',      -- { "_head.html": "...", "_books.html": "..." }
    manifest JSONB NOT NULL DEFAULT '{}',           -- tokens, capabilities, metadata
    preview_image_url TEXT,                 -- template thumbnail
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    
    -- Ensure unique template versions per tenant
    UNIQUE (COALESCE(tenant_id, '00000000-0000-0000-0000-000000000000'::uuid), name, version)
);

-- Add template-related columns to existing pages table
ALTER TABLE pages 
ADD COLUMN IF NOT EXISTS template_id UUID REFERENCES templates(id),
ADD COLUMN IF NOT EXISTS template_version INT DEFAULT 1,
ADD COLUMN IF NOT EXISTS draft_composition JSONB,  -- Puck composition JSON
ADD COLUMN IF NOT EXISTS published_url TEXT,       -- Static HTML URL after publish
ADD COLUMN IF NOT EXISTS published_etag TEXT,      -- For cache invalidation
ADD COLUMN IF NOT EXISTS is_published BOOLEAN DEFAULT false,
ADD COLUMN IF NOT EXISTS preview_image_url TEXT,   -- Page thumbnail
ADD COLUMN IF NOT EXISTS preview_status VARCHAR(20) DEFAULT 'queued'; -- queued|rendering|ready|failed

-- Create indexes for performance
CREATE INDEX IF NOT EXISTS idx_templates_tenant_name ON templates(tenant_id, name);
CREATE INDEX IF NOT EXISTS idx_templates_active ON templates(is_active) WHERE is_active = true;
CREATE INDEX IF NOT EXISTS idx_pages_template ON pages(template_id, template_version);
CREATE INDEX IF NOT EXISTS idx_pages_published ON pages(is_published) WHERE is_published = true;
CREATE INDEX IF NOT EXISTS idx_pages_site_slug ON pages(site_id, slug);

-- Insert a default literary template for authors
INSERT INTO templates (
    id,
    tenant_id,
    name,
    version,
    display_name,
    description,
    main_name,
    html_main,
    html_partials,
    manifest
) VALUES (
    '11111111-1111-1111-1111-111111111111'::uuid,
    NULL, -- Global template
    'literary-classic',
    1,
    'Literary Classic',
    'A beautiful, minimalist template perfect for authors and writers',
    'author.html',
    '<!DOCTYPE html>
<html lang="en">
<head>
    {% include "_head.html" %}
</head>
<body class="font-serif bg-cream text-charcoal">
    <main>
        {% for block in content %}
            {% if block.type == "HeroBlock" %}
                {% include "_hero.html" %}
            {% elif block.type == "TextBlock" %}
                {% include "_text.html" %}
            {% elif block.type == "CardBlock" %}
                {% include "_card.html" %}
            {% elif block.type == "SectionBlock" %}
                {% include "_section.html" %}
            {% endif %}
        {% endfor %}
    </main>
    {% include "_footer.html" %}
</body>
</html>',
    '{
        "_head.html": "<meta charset=\"UTF-8\"><meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\"><title>{{ page_title | default(\"Author Website\") }}</title><style>body{font-family:Georgia,serif;background:#fefefe;color:#2d3748;line-height:1.6}</style>",
        "_hero.html": "<section class=\"hero py-20 text-center\"><div class=\"container mx-auto px-4\"><h1 class=\"text-5xl font-bold mb-4\">{{ block.props.title }}</h1><p class=\"text-xl mb-8\">{{ block.props.subtitle }}</p><a href=\"{{ block.props.buttonHref }}\" class=\"bg-green-600 text-white px-8 py-3 rounded-lg hover:bg-green-700\">{{ block.props.buttonText }}</a></div></section>",
        "_text.html": "<div class=\"prose max-w-none mb-8\"><p>{{ block.props.children }}</p></div>",
        "_card.html": "<div class=\"card bg-white rounded-lg shadow-md overflow-hidden mb-6\"><img src=\"{{ block.props.imageUrl }}\" alt=\"{{ block.props.title }}\" class=\"w-full h-48 object-cover\"><div class=\"p-6\"><h3 class=\"text-xl font-bold mb-2\">{{ block.props.title }}</h3><p class=\"text-gray-600\">{{ block.props.content }}</p></div></div>",
        "_section.html": "<section class=\"py-12\" style=\"background-color: {{ block.props.backgroundColor }}; padding: {{ block.props.padding }}px 20px;\"><div class=\"container mx-auto\">{{ block.children | safe }}</div></section>",
        "_footer.html": "<footer class=\"bg-gray-100 py-8 mt-12\"><div class=\"container mx-auto px-4 text-center\"><p class=\"text-gray-600\">© {{ current_year }} {{ site_name }}. Powered by QuillSpace.</p></div></footer>"
    }',
    '{
        "tokens": {
            "colors": {
                "primary": "#10b981",
                "secondary": "#6b7280",
                "background": "#fefefe",
                "text": "#2d3748"
            },
            "fonts": {
                "heading": "Georgia, serif",
                "body": "Georgia, serif"
            }
        },
        "capabilities": {
            "darkMode": false,
            "responsive": true,
            "seoOptimized": true
        },
        "supports": ["HeroBlock", "TextBlock", "CardBlock", "SectionBlock", "ImageBlock", "ButtonBlock"]
    }'
) ON CONFLICT (COALESCE(tenant_id, '00000000-0000-0000-0000-000000000000'::uuid), name, version) DO NOTHING;

-- Update existing pages to use the default template if they don't have one
UPDATE pages 
SET 
    template_id = '11111111-1111-1111-1111-111111111111'::uuid,
    template_version = 1,
    draft_composition = COALESCE(draft_composition, '{"content": [], "root": {"props": {"title": "My Page"}}}')
WHERE template_id IS NULL;

-- Make template_id NOT NULL after setting defaults
ALTER TABLE pages ALTER COLUMN template_id SET NOT NULL;
ALTER TABLE pages ALTER COLUMN draft_composition SET NOT NULL;

-- Add RLS policies for templates
ALTER TABLE templates ENABLE ROW LEVEL SECURITY;

-- Templates policy: users can see global templates + their tenant's templates
CREATE POLICY templates_tenant_isolation ON templates
    FOR ALL
    USING (
        tenant_id IS NULL OR  -- Global templates
        tenant_id = COALESCE(
            current_setting('app.current_tenant_id', true)::uuid,
            '00000000-0000-0000-0000-000000000000'::uuid
        )
    );

-- Grant permissions
GRANT SELECT, INSERT, UPDATE, DELETE ON templates TO quillspace;
GRANT USAGE, SELECT ON ALL SEQUENCES IN SCHEMA public TO quillspace;

-- Add helpful comments
COMMENT ON TABLE templates IS 'Versioned MiniJinja templates for rendering pages';
COMMENT ON COLUMN templates.tenant_id IS 'NULL for global templates, UUID for tenant-specific templates';
COMMENT ON COLUMN templates.html_main IS 'Main MiniJinja template content';
COMMENT ON COLUMN templates.html_partials IS 'JSON object containing partial templates';
COMMENT ON COLUMN templates.manifest IS 'Template metadata including supported components and styling tokens';
COMMENT ON COLUMN pages.draft_composition IS 'Puck editor composition JSON - source of truth for page content';
COMMENT ON COLUMN pages.template_id IS 'Reference to template used for rendering this page';
