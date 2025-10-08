# QuillSpace Documentation

## Overview

QuillSpace is a multi-tenant publishing platform with an integrated drag-and-drop web builder, designed for authors and content creators. This documentation covers the complete system architecture, from the core publishing platform to the advanced web builder capabilities.

## Documentation Structure

### 📚 **Core Platform Documentation**
1. **[Architecture Guide](./architecture.md)** - Core platform architecture and technology decisions
2. **[Multi-Tenancy Guide](./multi-tenancy.md)** - Multi-tenant design patterns and RLS implementation
3. **[API Documentation](./api.md)** - RESTful API reference for the core platform
4. **[Development Guide](./development.md)** - Local development setup and workflows
5. **[Deployment Guide](./deployment.md)** - Production deployment strategies

### 🏗️ **Web Builder Documentation**
6. **[Web Builder Architecture](./WEB_BUILDER_ARCHITECTURE.md)** - High-level web builder design
7. **[Engineering Architecture](./ENGINEERING_ARCHITECTURE.md)** - Detailed technical implementation
8. **[Implementation Workplan](./IMPLEMENTATION_WORKPLAN.md)** - 24-week development roadmap

## System Overview

### **Current State: Core Publishing Platform**
- ✅ Multi-tenant content management system
- ✅ JWT authentication with Casbin RBAC
- ✅ PostgreSQL with Row-Level Security
- ✅ ClickHouse analytics pipeline
- ✅ Real user data (Yasin Kakande, Josephine Nakimuli)

### **Future State: Integrated Web Builder**
- 🚧 Drag-and-drop website creation
- 🚧 MiniJinja template engine with database loading
- 🚧 Puck visual editor integration
- 🚧 Qwik SSR/SSG for SEO optimization
- 🚧 Custom domain management with automated SSL
- 🚧 Widget marketplace and extensibility

## Technology Stack

### **Backend (Rust)**
```rust
// Core Infrastructure
- Axum HTTP framework
- tokio-postgres connection pool
- Casbin authorization
- JWT authentication
- ClickHouse analytics

// Web Builder Extensions
- MiniJinja template engine
- Domain verification service
- SSL certificate automation
- Asset management (MinIO)
```

### **Frontend**
```typescript
// Current: Basic API interface
// Future: Qwik + Puck visual editor
- Qwik City for SSR/SSG
- Puck editor via qwikify$
- Component marketplace
- Real-time preview system
```

### **Infrastructure**
```yaml
# Current: Docker Compose
# Future: Kubernetes with auto-scaling
- PostgreSQL (primary database)
- ClickHouse (analytics)
- Redis (caching)
- MinIO (object storage)
- Caddy (reverse proxy + SSL)
```

## Key Architectural Decisions

### **1. Multi-Tenancy Strategy**
- **Shared Schema + RLS**: Single database with row-level security
- **Tenant Context**: Session-based tenant isolation
- **UUID-based IDs**: Secure, non-enumerable identifiers

### **2. Template System**
- **Runtime Compilation**: MiniJinja with database loader
- **Versioning**: Template history and migration support
- **Multi-Tenant**: Per-tenant templates with public marketplace

### **3. Visual Editor**
- **Open Source**: Puck (MIT licensed) vs proprietary Builder.io
- **React Integration**: qwikify$ for Qwik compatibility
- **JSON Composition**: Serializable page structure

### **4. SEO & Performance**
- **Resumable Hydration**: Qwik's zero-hydration approach
- **Pre-rendering**: SSG for static pages, SSR for dynamic
- **Edge Caching**: CDN with intelligent invalidation

### **5. Domain Management**
- **DNS Verification**: TXT record validation
- **Automated SSL**: Caddy On-Demand TLS with ACME
- **Host-based Routing**: Multi-domain support

## Implementation Phases

### **Phase 1: Foundation (Weeks 1-4)**
- Database schema with RLS policies
- MiniJinja template engine integration
- Basic site and page management APIs
- Template CRUD with versioning

### **Phase 2: Visual Editor (Weeks 5-8)**
- Qwik frontend setup with Puck integration
- Component library and widget system
- Live preview and editing interface
- Template selection and customization

### **Phase 3: Domain & TLS (Weeks 9-12)**
- Domain verification service
- Caddy On-Demand TLS setup
- Custom domain onboarding flow
- SSL certificate monitoring

### **Phase 4: SEO & Performance (Weeks 13-16)**
- Qwik SSR/SSG implementation
- SEO metadata and structured data
- Sitemap and robots.txt generation
- Performance optimization

### **Phase 5: Advanced Features (Weeks 17-20)**
- Widget marketplace development
- External data source integration
- Analytics dashboard
- Security audit

### **Phase 6: Production (Weeks 21-24)**
- Kubernetes deployment
- CI/CD pipeline
- Load testing
- Production launch

## Getting Started

### **For Developers**
1. Read [Development Guide](./development.md) for local setup
2. Review [Architecture Guide](./architecture.md) for system understanding
3. Check [Multi-Tenancy Guide](./multi-tenancy.md) for data isolation patterns

### **For DevOps**
1. Review [Deployment Guide](./deployment.md) for production setup
2. Check [Engineering Architecture](./ENGINEERING_ARCHITECTURE.md) for infrastructure requirements

### **For Product/Project Management**
1. Review [Implementation Workplan](./IMPLEMENTATION_WORKPLAN.md) for timeline and milestones
2. Check [Web Builder Architecture](./WEB_BUILDER_ARCHITECTURE.md) for feature overview

## Current Status

### **✅ Completed (Core Platform)**
- Multi-tenant database with RLS
- User authentication and authorization
- Content management APIs
- Analytics pipeline
- Real user data seeded

### **🚧 In Progress (Web Builder)**
- Database schema design ← **Current Phase**
- MiniJinja integration planning
- Puck editor evaluation

### **📋 Next Steps**
1. Apply web builder database schema
2. Integrate MiniJinja template engine
3. Build template management APIs
4. Set up Qwik frontend with Puck

## Contributing

### **Code Standards**
- Rust: Follow `rustfmt` and `clippy` recommendations
- TypeScript: ESLint + Prettier configuration
- SQL: Consistent naming and indexing patterns
- Documentation: Keep all docs updated with changes

### **Testing Requirements**
- Unit tests: 90%+ coverage for business logic
- Integration tests: API endpoint validation
- E2E tests: Critical user workflows
- Performance tests: Load and stress testing

## Support

### **Internal Team**
- Architecture questions: Review engineering docs
- Implementation help: Check development guide
- Deployment issues: Consult deployment guide

### **External Contributors**
- API usage: See API documentation
- Multi-tenancy: Review multi-tenancy guide
- Feature requests: Submit via GitHub issues

---

**Last Updated**: October 2025  
**Version**: 2.0 (Web Builder Integration)  
**Status**: Active Development
