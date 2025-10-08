# QuillSpace Multi-Tenant Web Builder Architecture

## Overview

QuillSpace will implement a comprehensive multi-tenant web builder platform that enables authors to create, customize, and publish SEO-optimized websites with drag-and-drop editing capabilities. This document outlines the enhanced architecture based on comprehensive research and audit findings.

## Core Architecture Principles

### 1. **Multi-Tenancy with Row-Level Security**
- PostgreSQL with RLS for data isolation
- UUID-based tenant identification
- Secure cross-tenant data prevention

### 2. **Dynamic Template System**
- Database-stored templates with versioning
- Runtime template loading and swapping
- Schema-based content migration

### 3. **SEO-First Rendering**
- Server-side pre-rendering for search engines
- Dynamic metadata generation
- Per-tenant sitemaps and robots.txt

### 4. **Open-Source Visual Editing**
- Vendor-independent editing experience
- Extensible widget marketplace
- JSON-based page composition

## Technology Stack

### Backend (Rust/Axum)
- **Template Engine:** minijinja with database loader
- **Authentication:** JWT with bcrypt
- **Authorization:** Casbin RBAC
- **Database:** PostgreSQL with RLS
- **Analytics:** ClickHouse

### Frontend (Qwik + React)
- **Visual Editor:** Puck (MIT licensed)
- **Rendering:** Qwik SSR for SEO
- **UI Framework:** Qwik with React components
- **State Management:** Qwik signals

### Infrastructure
- **Reverse Proxy:** Nginx/Caddy with ACME SSL
- **CDN:** Static HTML caching
- **Domain Management:** Automated DNS verification
- **Object Storage:** Template and asset storage

## Database Schema

### Templates System
```sql
-- Template definitions with versioning
CREATE TABLE templates (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id UUID REFERENCES tenants(id),
    name VARCHAR(255) NOT NULL,
    description TEXT,
    category VARCHAR(100),
    html_source TEXT NOT NULL, -- minijinja template
    default_schema JSONB NOT NULL, -- editable blocks definition
    preview_image_url TEXT,
    is_public BOOLEAN DEFAULT false,
    version INTEGER DEFAULT 1,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Template version history
CREATE TABLE template_versions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    template_id UUID REFERENCES templates(id),
    version INTEGER NOT NULL,
    html_source TEXT NOT NULL,
    schema JSONB NOT NULL,
    changelog TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW()
);
```

### Sites and Pages
```sql
-- Author websites
CREATE TABLE sites (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id UUID REFERENCES tenants(id),
    name VARCHAR(255) NOT NULL,
    template_id UUID REFERENCES templates(id),
    custom_domain VARCHAR(255),
    subdomain VARCHAR(100) UNIQUE, -- author.quillspace.com
    is_published BOOLEAN DEFAULT false,
    seo_settings JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Individual pages within sites
CREATE TABLE pages (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    site_id UUID REFERENCES sites(id),
    slug VARCHAR(200) NOT NULL,
    title VARCHAR(500) NOT NULL,
    meta_description TEXT,
    content JSONB NOT NULL, -- Puck JSON composition
    is_published BOOLEAN DEFAULT false,
    published_html TEXT, -- Pre-rendered HTML for SEO
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(site_id, slug)
);
```

### Domain Management
```sql
-- Custom domain verification
CREATE TABLE domains (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    site_id UUID REFERENCES sites(id),
    domain VARCHAR(255) NOT NULL UNIQUE,
    verification_token VARCHAR(255) NOT NULL,
    is_verified BOOLEAN DEFAULT false,
    ssl_certificate_id VARCHAR(255),
    dns_configured BOOLEAN DEFAULT false,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    verified_at TIMESTAMPTZ
);

-- SSL certificate management
CREATE TABLE ssl_certificates (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    domain VARCHAR(255) NOT NULL,
    certificate_pem TEXT NOT NULL,
    private_key_pem TEXT NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL,
    auto_renew BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT NOW()
);
```

### Widget System
```sql
-- Available widgets for the marketplace
CREATE TABLE widgets (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(255) NOT NULL,
    description TEXT,
    category VARCHAR(100),
    component_config JSONB NOT NULL, -- Puck component definition
    external_api_config JSONB, -- API integration settings
    is_public BOOLEAN DEFAULT true,
    created_by UUID REFERENCES users(id),
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Widget usage tracking
CREATE TABLE page_widgets (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    page_id UUID REFERENCES pages(id),
    widget_id UUID REFERENCES widgets(id),
    instance_config JSONB NOT NULL, -- User customization
    position INTEGER NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW()
);
```

## Core Components

### 1. Template Engine (minijinja)
```rust
use minijinja::{Environment, Source};
use serde_json::Value;

pub struct DatabaseTemplateLoader {
    db: Arc<DatabaseConnections>,
}

impl DatabaseTemplateLoader {
    pub async fn load_template(&self, name: &str, tenant_id: Uuid) -> Result<String> {
        let query = "SELECT html_source FROM templates WHERE name = $1 AND tenant_id = $2";
        let row = self.db.postgres.query_one(query, &[&name, &tenant_id]).await?;
        Ok(row.get("html_source"))
    }
    
    pub fn create_environment(&self) -> Environment {
        let mut env = Environment::new();
        env.set_loader(|name| {
            // Load template from database
            self.load_template(name, tenant_id).await
        });
        env
    }
}
```

### 2. Visual Editor Integration (Puck)
```typescript
// Puck configuration for QuillSpace
const puckConfig = {
  components: {
    Hero: {
      fields: {
        title: { type: "text" },
        subtitle: { type: "textarea" },
        backgroundImage: { type: "external", fetchList: fetchImages }
      },
      render: ({ title, subtitle, backgroundImage }) => (
        <section className="hero" style={{ backgroundImage: `url(${backgroundImage})` }}>
          <h1>{title}</h1>
          <p>{subtitle}</p>
        </section>
      )
    },
    BlogPosts: {
      fields: {
        category: { type: "select", options: fetchCategories },
        limit: { type: "number", defaultValue: 5 }
      },
      render: ({ category, limit }) => (
        <BlogPostList category={category} limit={limit} />
      )
    }
  }
};
```

### 3. Domain Verification Service
```rust
pub struct DomainVerificationService {
    dns_resolver: Arc<TrustDnsResolver>,
    acme_client: Arc<AcmeClient>,
}

impl DomainVerificationService {
    pub async fn verify_domain(&self, domain: &str, token: &str) -> Result<bool> {
        let txt_records = self.dns_resolver
            .txt_lookup(&format!("_quillspace-verify.{}", domain))
            .await?;
            
        for record in txt_records.iter() {
            if record.to_string().contains(token) {
                return Ok(true);
            }
        }
        Ok(false)
    }
    
    pub async fn issue_ssl_certificate(&self, domain: &str) -> Result<Certificate> {
        self.acme_client.new_order()
            .add_dns_name(domain)
            .finalize()
            .await
    }
}
```

### 4. SEO Optimization Service
```rust
pub struct SeoService {
    renderer: Arc<QwikRenderer>,
    template_engine: Arc<DatabaseTemplateLoader>,
}

impl SeoService {
    pub async fn pre_render_page(&self, page: &Page, site: &Site) -> Result<String> {
        // Load template and render with page content
        let template = self.template_engine
            .load_template(&site.template_name, site.tenant_id)
            .await?;
            
        let html = self.renderer
            .render_to_string(template, &page.content)
            .await?;
            
        // Add SEO metadata
        let optimized_html = self.add_seo_metadata(html, page, site).await?;
        
        Ok(optimized_html)
    }
    
    async fn add_seo_metadata(&self, html: String, page: &Page, site: &Site) -> Result<String> {
        // Inject meta tags, structured data, canonical URLs
        let meta_tags = format!(
            r#"<title>{}</title>
               <meta name="description" content="{}">
               <meta property="og:title" content="{}">
               <link rel="canonical" href="https://{}/{}">
               <script type="application/ld+json">{}</script>"#,
            page.title,
            page.meta_description.as_deref().unwrap_or(""),
            page.title,
            site.custom_domain.as_deref().unwrap_or(&format!("{}.quillspace.com", site.subdomain)),
            page.slug,
            self.generate_structured_data(page, site)?
        );
        
        Ok(html.replace("</head>", &format!("{}</head>", meta_tags)))
    }
}
```

## Implementation Roadmap

### Phase 1: Foundation (Weeks 1-4)
- [ ] Database schema implementation
- [ ] minijinja template engine integration
- [ ] Basic template CRUD operations
- [ ] Template versioning system
- [ ] Row-level security for templates

### Phase 2: Visual Editor (Weeks 5-8)
- [ ] Puck integration in Qwik frontend
- [ ] Component library development
- [ ] JSON page composition storage
- [ ] Live preview functionality
- [ ] Template selection interface

### Phase 3: Domain Management (Weeks 9-12)
- [ ] DNS verification service
- [ ] ACME SSL certificate automation
- [ ] Reverse proxy configuration
- [ ] Custom domain onboarding flow
- [ ] Host-based routing middleware

### Phase 4: SEO & Rendering (Weeks 13-16)
- [ ] Qwik SSR integration
- [ ] Pre-rendering service
- [ ] Dynamic metadata generation
- [ ] Sitemap and robots.txt generation
- [ ] CDN integration for static assets

### Phase 5: Widget Marketplace (Weeks 17-20)
- [ ] Widget system architecture
- [ ] External data source integration
- [ ] Plugin API development
- [ ] Widget marketplace UI
- [ ] Third-party API connectors

### Phase 6: Advanced Features (Weeks 21-24)
- [ ] Template migration system
- [ ] A/B testing framework
- [ ] Analytics integration
- [ ] Performance monitoring
- [ ] Mobile responsive editing

## Security Considerations

### 1. **Template Security**
- Sandbox minijinja execution
- Input validation for template variables
- XSS prevention in user content
- Content Security Policy headers

### 2. **Domain Verification**
- Secure token generation
- DNS poisoning protection
- Certificate validation
- Rate limiting for verification attempts

### 3. **Widget Security**
- External API rate limiting
- Content sanitization
- CORS policy enforcement
- Widget permission system

## Performance Optimization

### 1. **Caching Strategy**
- Template compilation caching
- Pre-rendered HTML caching
- CDN integration for static assets
- Database query optimization

### 2. **Rendering Performance**
- Qwik's resumable hydration
- Progressive loading of components
- Image optimization and lazy loading
- Critical CSS inlining

## Monitoring and Analytics

### 1. **System Metrics**
- Template rendering performance
- Domain verification success rates
- SSL certificate renewal status
- Widget usage analytics

### 2. **User Analytics**
- Page view tracking per tenant
- Editor usage patterns
- Template popularity metrics
- Performance core web vitals

## Conclusion

This architecture provides a comprehensive foundation for building a multi-tenant web builder that rivals commercial solutions while maintaining full control over the technology stack. The emphasis on open-source tools, SEO optimization, and extensibility ensures long-term viability and competitive advantage.

The phased implementation approach allows for iterative development and early user feedback, while the modular architecture supports future enhancements and scaling requirements.
