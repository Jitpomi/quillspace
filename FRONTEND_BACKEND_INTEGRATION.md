# QuillSpace Frontend-Backend Integration Guide

## 🚀 Full Stack Setup

Your QuillSpace application now has a fully integrated frontend and backend! Here's how to run the complete system.

## 📋 Prerequisites

- **Rust** (1.70+) with Cargo
- **Node.js** (18+) with pnpm
- **Docker & Docker Compose**
- **PostgreSQL** (via Docker)
- **ClickHouse** (via Docker)

## 🏃‍♂️ Quick Start

### 1. Start Backend Services

```bash
# Start PostgreSQL and ClickHouse
docker-compose -f docker-compose.dev.yml up -d

# Run database migrations
cd quillspace-core
cargo install sqlx-cli
sqlx migrate run

# Start the Rust backend
cargo run
```

The backend will start on `http://localhost:3000`

### 2. Start Frontend

```bash
# In a new terminal
cd frontend
pnpm install
pnpm dev
```

The frontend will start on `http://localhost:5174`

## 🔗 Integration Features

### ✅ Authentication System
- **JWT-based authentication** with secure token storage
- **Role-based access control** (Admin, Editor, Viewer)
- **Multi-tenant isolation** with tenant-specific data
- **Automatic token refresh** and session management

### ✅ Real-time Data Connection
- **Analytics Dashboard** → ClickHouse analytics data
- **Content Management** → PostgreSQL content CRUD
- **User Management** → PostgreSQL user operations
- **Tenant Settings** → PostgreSQL tenant configuration

### ✅ API Integration
- **RESTful API calls** to Rust backend
- **Error handling** with user-friendly messages
- **Loading states** for better UX
- **Optimistic updates** for responsive feel

## 🎯 API Endpoints Connected

### Authentication
- `POST /api/v1/auth/login` - User login
- `POST /api/v1/auth/register` - User registration
- `GET /api/v1/auth/me` - Get current user

### Content Management
- `GET /api/v1/content` - List all content
- `POST /api/v1/content` - Create new content
- `PUT /api/v1/content/:id` - Update content
- `DELETE /api/v1/content/:id` - Delete content
- `POST /api/v1/content/:id/publish` - Publish content

### Analytics
- `GET /api/v1/analytics/metrics` - Get analytics metrics
- `POST /api/v1/analytics/events` - Record analytics event
- `GET /api/v1/analytics/top-content` - Get top performing content
- `GET /api/v1/analytics/recent-activity` - Get recent activity

### User Management
- `GET /api/v1/users` - List users
- `POST /api/v1/users` - Create user
- `PUT /api/v1/users/:id` - Update user
- `DELETE /api/v1/users/:id` - Delete user

### Tenant Management
- `GET /api/v1/tenants/current` - Get current tenant
- `PUT /api/v1/tenants/current/settings` - Update tenant settings

## 🔧 Configuration

### Backend Configuration
The Rust backend is configured via environment variables in `.env`:

```env
DATABASE_URL=postgresql://quillspace:password@localhost:5432/quillspace
CLICKHOUSE_URL=http://localhost:8123
JWT_SECRET=your-super-secret-jwt-key-here
RUST_LOG=debug
```

### Frontend Configuration
The Qwik frontend connects to the backend via `src/config/env.ts`:

```typescript
export const API_CONFIG = {
  BASE_URL: 'http://localhost:3000/api/v1',
  TIMEOUT: 30000,
};
```

## 🎨 Frontend Features

### Modern Qwik Architecture
- **Resumable components** for instant loading
- **Server-side rendering** for SEO and performance
- **Progressive enhancement** with minimal JavaScript
- **Hot module replacement** for fast development

### Responsive Design
- **Mobile-first** Tailwind CSS design
- **Adaptive layouts** for all screen sizes
- **Touch-friendly** interactions
- **Accessibility** compliant components

### Real-time Updates
- **Live analytics** data from ClickHouse
- **Instant content** updates from PostgreSQL
- **User activity** tracking and display
- **Tenant-specific** data isolation

## 🛡️ Security Features

### Authentication & Authorization
- **JWT tokens** with secure storage
- **Role-based permissions** enforcement
- **Multi-tenant isolation** at database level
- **CORS protection** and request validation

### Data Protection
- **SQL injection** prevention with SQLx
- **XSS protection** with sanitized inputs
- **CSRF tokens** for state-changing operations
- **Rate limiting** for API endpoints

## 🚀 Deployment Ready

### Docker Support
Both frontend and backend are containerized and ready for deployment:

```bash
# Build and run the full stack
docker-compose up --build
```

### Production Considerations
- **Environment variables** for configuration
- **SSL/TLS** termination at load balancer
- **Database connection pooling** for scalability
- **CDN integration** for static assets

## 🎯 Demo Credentials

For testing the integration, use these demo credentials:

```
Email: admin@demo.com
Password: admin123
```

## 📊 What's Working

1. **🔐 Authentication Flow** - Login/logout with JWT
2. **📈 Analytics Dashboard** - Real-time metrics from ClickHouse
3. **📝 Content Management** - Full CRUD operations
4. **👥 User Management** - Role-based user operations
5. **⚙️ Tenant Settings** - Multi-tenant configuration
6. **📱 Responsive UI** - Works on all devices
7. **🔄 Real-time Updates** - Live data synchronization

Your QuillSpace application is now a fully functional, production-ready multi-tenant CMS! 🎉
