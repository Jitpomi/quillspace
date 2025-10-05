# QuillSpace Development Guide

## Overview

This guide covers setting up a local development environment for QuillSpace, including all necessary tools, dependencies, and development workflows. QuillSpace uses a modern Rust + Qwik stack optimized for developer productivity.

## Prerequisites

### System Requirements

| Component | Minimum | Recommended | Notes |
|-----------|---------|-------------|-------|
| **OS** | macOS 10.15+, Ubuntu 20.04+, Windows 10+ | Latest stable | WSL2 recommended for Windows |
| **RAM** | 8GB | 16GB+ | ClickHouse and compilation require memory |
| **Storage** | 20GB free | 50GB+ SSD | Fast storage improves build times |
| **CPU** | 2 cores | 4+ cores | Rust compilation benefits from parallelism |

### Required Software

#### 1. Rust Toolchain

```bash
# Install Rust via rustup
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Verify installation
rustc --version
cargo --version

# Install additional components
rustup component add rustfmt clippy
cargo install cargo-watch cargo-audit
```

#### 2. Node.js & Package Manager

```bash
# Install Node.js (via nvm recommended)
curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.0/install.sh | bash
nvm install 18
nvm use 18

# Verify installation
node --version
npm --version

# Install pnpm (recommended for Qwik)
npm install -g pnpm
```

#### 3. Database Tools

```bash
# PostgreSQL client
# macOS
brew install postgresql

# Ubuntu
sudo apt-get install postgresql-client

# Windows (via Chocolatey)
choco install postgresql

# ClickHouse client
# macOS
brew install clickhouse

# Ubuntu
curl https://clickhouse.com/ | sh

# Verify installations
psql --version
clickhouse-client --version
```

#### 4. Container Tools

```bash
# Docker Desktop (recommended)
# Download from https://www.docker.com/products/docker-desktop

# Or Docker Engine (Linux)
curl -fsSL https://get.docker.com -o get-docker.sh
sh get-docker.sh

# Docker Compose
sudo curl -L "https://github.com/docker/compose/releases/latest/download/docker-compose-$(uname -s)-$(uname -m)" -o /usr/local/bin/docker-compose
sudo chmod +x /usr/local/bin/docker-compose

# Verify installation
docker --version
docker-compose --version
```

#### 5. Development Tools

```bash
# Git
git --version  # Usually pre-installed

# Code editor extensions (VS Code recommended)
code --install-extension rust-lang.rust-analyzer
code --install-extension bradlc.vscode-tailwindcss
code --install-extension ms-vscode.vscode-typescript-next

# Optional: Database GUI tools
# - TablePlus (macOS/Windows)
# - DBeaver (cross-platform)
# - pgAdmin (PostgreSQL)
```

## Project Setup

### 1. Clone Repository

```bash
# Clone the repository
git clone <repository-url>
cd quillspace

# Check project structure
tree -L 2
```

### 2. Environment Configuration

```bash
# Copy environment template
cp .env.example .env

# Edit environment variables
nano .env
```

Example `.env` file:

```bash
# Database Configuration
DATABASE_URL=postgresql://quillspace:dev_password@localhost:5432/quillspace_dev
CLICKHOUSE_URL=http://localhost:8123/analytics_dev
REDIS_URL=redis://localhost:6379

# Application Configuration
JWT_SECRET=your_jwt_secret_key_minimum_32_characters_for_dev
API_BASE_URL=http://localhost:3000
FRONTEND_URL=http://localhost:5173

# Development Settings
RUST_LOG=debug
RUST_BACKTRACE=1
NODE_ENV=development

# Feature Flags
ENABLE_METRICS=true
ENABLE_TRACING=true
ENABLE_HOT_RELOAD=true
```

### 3. Start Infrastructure Services

```bash
# Start databases and cache
docker-compose -f docker-compose.dev.yml up -d

# Verify services are running
docker-compose ps
```

Example `docker-compose.dev.yml`:

```yaml
version: '3.8'

services:
  postgres-dev:
    image: postgres:15-alpine
    environment:
      POSTGRES_DB: quillspace_dev
      POSTGRES_USER: quillspace
      POSTGRES_PASSWORD: dev_password
    ports:
      - "5432:5432"
    volumes:
      - postgres_dev_data:/var/lib/postgresql/data
      - ./scripts/init-dev.sql:/docker-entrypoint-initdb.d/init.sql

  clickhouse-dev:
    image: clickhouse/clickhouse-server:23-alpine
    environment:
      CLICKHOUSE_DB: analytics_dev
      CLICKHOUSE_USER: quillspace
      CLICKHOUSE_PASSWORD: dev_password
    ports:
      - "8123:8123"
      - "9000:9000"
    volumes:
      - clickhouse_dev_data:/var/lib/clickhouse

  redis-dev:
    image: redis:7-alpine
    ports:
      - "6379:6379"
    volumes:
      - redis_dev_data:/data

volumes:
  postgres_dev_data:
  clickhouse_dev_data:
  redis_dev_data:
```

### 4. Database Setup

```bash
# Run database migrations
cargo run --bin migrate

# Seed development data
cargo run --bin seed

# Verify database setup
psql $DATABASE_URL -c "SELECT count(*) FROM tenants;"
```

## Development Workflow

### 1. Backend Development

#### Project Structure

```
quillspace-core/
├── src/
│   ├── main.rs              # Application entry point
│   ├── lib.rs               # Library root
│   ├── config.rs            # Configuration management
│   ├── routes/              # API route handlers
│   │   ├── mod.rs
│   │   ├── auth.rs          # Authentication routes
│   │   ├── tenants.rs       # Tenant management
│   │   ├── users.rs         # User management
│   │   └── content.rs       # Content management
│   ├── models/              # Data models
│   │   ├── mod.rs
│   │   ├── tenant.rs
│   │   ├── user.rs
│   │   └── content.rs
│   ├── services/            # Business logic
│   │   ├── mod.rs
│   │   ├── auth_service.rs
│   │   ├── tenant_service.rs
│   │   └── content_service.rs
│   ├── middleware/          # Custom middleware
│   │   ├── mod.rs
│   │   ├── auth.rs
│   │   ├── tenant.rs
│   │   └── logging.rs
│   └── utils/               # Utility functions
│       ├── mod.rs
│       ├── database.rs
│       └── validation.rs
├── migrations/              # Database migrations
├── tests/                   # Integration tests
└── Cargo.toml
```

#### Running the Backend

```bash
# Development server with hot reload
cargo watch -x 'run --bin quillspace-core'

# Or run directly
cargo run --bin quillspace-core

# Run with specific log level
RUST_LOG=debug cargo run --bin quillspace-core

# Run tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_create_tenant

# Check code formatting
cargo fmt --check

# Run linter
cargo clippy -- -D warnings

# Security audit
cargo audit
```

#### Adding New Routes

1. **Define the route handler**:

```rust
// src/routes/example.rs
use axum::{extract::Path, http::StatusCode, response::Json, Extension};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Deserialize)]
pub struct CreateExampleRequest {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Serialize)]
pub struct ExampleResponse {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

pub async fn create_example(
    Extension(tenant_id): Extension<Uuid>,
    Json(payload): Json<CreateExampleRequest>,
) -> Result<Json<ExampleResponse>, StatusCode> {
    // Implementation here
    todo!()
}

pub async fn get_example(
    Path(id): Path<Uuid>,
    Extension(tenant_id): Extension<Uuid>,
) -> Result<Json<ExampleResponse>, StatusCode> {
    // Implementation here
    todo!()
}
```

2. **Register the route**:

```rust
// src/main.rs
use axum::{routing::{get, post}, Router};

let app = Router::new()
    .route("/api/examples", post(routes::example::create_example))
    .route("/api/examples/:id", get(routes::example::get_example));
```

#### Database Migrations

```bash
# Create new migration
cargo run --bin create-migration -- create_examples_table

# This creates: migrations/TIMESTAMP_create_examples_table.sql
```

Example migration:

```sql
-- migrations/20231201120000_create_examples_table.sql
-- Up
CREATE TABLE examples (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    name VARCHAR NOT NULL,
    description TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Enable RLS
ALTER TABLE examples ENABLE ROW LEVEL SECURITY;

-- Create tenant isolation policy
CREATE POLICY tenant_isolation ON examples
    USING (tenant_id = current_setting('app.current_tenant_id')::UUID);

-- Create indexes
CREATE INDEX idx_examples_tenant_id ON examples(tenant_id);
CREATE INDEX idx_examples_created_at ON examples(created_at);

-- Down
DROP TABLE IF EXISTS examples;
```

### 2. Frontend Development

#### Project Structure

```
frontend/
├── src/
│   ├── routes/              # File-based routing
│   │   ├── layout.tsx       # Root layout
│   │   ├── index.tsx        # Home page
│   │   ├── login/
│   │   │   └── index.tsx    # Login page
│   │   └── dashboard/
│   │       ├── layout.tsx   # Dashboard layout
│   │       └── index.tsx    # Dashboard page
│   ├── components/          # Reusable components
│   │   ├── ui/              # Base UI components
│   │   │   ├── button.tsx
│   │   │   ├── input.tsx
│   │   │   └── modal.tsx
│   │   ├── forms/           # Form components
│   │   └── layout/          # Layout components
│   ├── services/            # API client services
│   │   ├── api.ts           # Base API client
│   │   ├── auth.ts          # Authentication API
│   │   └── content.ts       # Content API
│   ├── stores/              # State management
│   │   ├── auth.ts          # Auth state
│   │   └── tenant.ts        # Tenant state
│   └── utils/               # Utility functions
│       ├── validation.ts
│       └── formatting.ts
├── public/                  # Static assets
├── package.json
└── vite.config.ts
```

#### Running the Frontend

```bash
# Navigate to frontend directory
cd frontend

# Install dependencies
pnpm install

# Start development server
pnpm dev

# Build for production
pnpm build

# Preview production build
pnpm preview

# Run tests
pnpm test

# Run linter
pnpm lint

# Format code
pnpm format
```

#### Creating Components

```tsx
// src/components/ui/button.tsx
import { component$, Slot } from '@builder.io/qwik';
import { type QwikIntrinsicElements } from '@builder.io/qwik';

interface ButtonProps extends QwikIntrinsicElements['button'] {
  variant?: 'primary' | 'secondary' | 'danger';
  size?: 'sm' | 'md' | 'lg';
}

export const Button = component$<ButtonProps>(({ 
  variant = 'primary', 
  size = 'md', 
  class: className,
  ...props 
}) => {
  const baseClasses = 'font-medium rounded-lg transition-colors';
  const variantClasses = {
    primary: 'bg-blue-600 text-white hover:bg-blue-700',
    secondary: 'bg-gray-200 text-gray-900 hover:bg-gray-300',
    danger: 'bg-red-600 text-white hover:bg-red-700',
  };
  const sizeClasses = {
    sm: 'px-3 py-1.5 text-sm',
    md: 'px-4 py-2 text-base',
    lg: 'px-6 py-3 text-lg',
  };

  return (
    <button
      class={`${baseClasses} ${variantClasses[variant]} ${sizeClasses[size]} ${className || ''}`}
      {...props}
    >
      <Slot />
    </button>
  );
});
```

#### API Integration

```typescript
// src/services/api.ts
import { server$ } from '@builder.io/qwik-city';

export const api = {
  baseURL: import.meta.env.VITE_API_URL || 'http://localhost:3000',
  
  async request<T>(endpoint: string, options: RequestInit = {}): Promise<T> {
    const url = `${this.baseURL}${endpoint}`;
    const token = localStorage.getItem('auth_token');
    
    const response = await fetch(url, {
      ...options,
      headers: {
        'Content-Type': 'application/json',
        ...(token && { Authorization: `Bearer ${token}` }),
        ...options.headers,
      },
    });

    if (!response.ok) {
      throw new Error(`API Error: ${response.status}`);
    }

    return response.json();
  },

  get<T>(endpoint: string): Promise<T> {
    return this.request<T>(endpoint);
  },

  post<T>(endpoint: string, data: any): Promise<T> {
    return this.request<T>(endpoint, {
      method: 'POST',
      body: JSON.stringify(data),
    });
  },
};

// Server-side API calls
export const getUser = server$(async (userId: string) => {
  // This runs on the server
  const user = await api.get(`/api/users/${userId}`);
  return user;
});
```

### 3. Testing

#### Backend Testing

```rust
// tests/integration_test.rs
use axum::http::StatusCode;
use serde_json::json;
use tower::ServiceExt;

#[tokio::test]
async fn test_create_tenant() {
    let app = create_test_app().await;
    
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/tenants")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "name": "Test Tenant",
                        "domain": "test.example.com"
                    }).to_string()
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);
}

async fn create_test_app() -> Router {
    // Setup test database and return app
    todo!()
}
```

#### Frontend Testing

```typescript
// src/components/button.test.tsx
import { createDOM } from '@builder.io/qwik/testing';
import { test, expect } from 'vitest';
import { Button } from './button';

test('Button renders correctly', async () => {
  const { screen, render } = await createDOM();
  
  await render(<Button>Click me</Button>);
  
  const button = screen.querySelector('button');
  expect(button).toBeTruthy();
  expect(button?.textContent).toBe('Click me');
});
```

### 4. Code Quality

#### Pre-commit Hooks

Create `.pre-commit-config.yaml`:

```yaml
repos:
  - repo: local
    hooks:
      - id: cargo-fmt
        name: cargo fmt
        entry: cargo fmt --all --
        language: system
        types: [rust]
        
      - id: cargo-clippy
        name: cargo clippy
        entry: cargo clippy --all-targets --all-features -- -D warnings
        language: system
        types: [rust]
        
      - id: cargo-test
        name: cargo test
        entry: cargo test
        language: system
        types: [rust]
        pass_filenames: false
        
      - id: frontend-lint
        name: frontend lint
        entry: bash -c 'cd frontend && pnpm lint'
        language: system
        types: [typescript]
        pass_filenames: false
```

Install pre-commit:

```bash
pip install pre-commit
pre-commit install
```

#### VS Code Configuration

Create `.vscode/settings.json`:

```json
{
  "rust-analyzer.cargo.features": "all",
  "rust-analyzer.checkOnSave.command": "clippy",
  "editor.formatOnSave": true,
  "editor.codeActionsOnSave": {
    "source.fixAll": true
  },
  "[rust]": {
    "editor.defaultFormatter": "rust-lang.rust-analyzer"
  },
  "[typescript]": {
    "editor.defaultFormatter": "esbenp.prettier-vscode"
  },
  "files.associations": {
    "*.rs": "rust"
  }
}
```

## Debugging

### Backend Debugging

```rust
// Add debug logging
use tracing::{debug, info, error};

#[tracing::instrument(skip(db))]
pub async fn create_tenant(
    Json(payload): Json<CreateTenantRequest>,
    Extension(db): Extension<PgPool>,
) -> Result<Json<TenantResponse>, StatusCode> {
    debug!("Creating tenant with payload: {:?}", payload);
    
    match tenant_service::create(&db, payload).await {
        Ok(tenant) => {
            info!("Tenant created successfully: {}", tenant.id);
            Ok(Json(tenant.into()))
        }
        Err(e) => {
            error!("Failed to create tenant: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
```

### Database Debugging

```bash
# Connect to development database
psql $DATABASE_URL

# Check current connections
SELECT * FROM pg_stat_activity WHERE datname = 'quillspace_dev';

# Monitor slow queries
SELECT query, mean_exec_time, calls 
FROM pg_stat_statements 
ORDER BY mean_exec_time DESC 
LIMIT 10;

# Check table sizes
SELECT 
    schemaname,
    tablename,
    pg_size_pretty(pg_total_relation_size(schemaname||'.'||tablename)) as size
FROM pg_tables 
WHERE schemaname = 'public'
ORDER BY pg_total_relation_size(schemaname||'.'||tablename) DESC;
```

### Frontend Debugging

```typescript
// Debug API calls
const debugApi = {
  ...api,
  async request<T>(endpoint: string, options: RequestInit = {}): Promise<T> {
    console.log(`API Request: ${options.method || 'GET'} ${endpoint}`);
    console.log('Options:', options);
    
    try {
      const result = await api.request<T>(endpoint, options);
      console.log('API Response:', result);
      return result;
    } catch (error) {
      console.error('API Error:', error);
      throw error;
    }
  }
};
```

## Performance Optimization

### Backend Optimization

```rust
// Connection pooling
let pool = PgPoolOptions::new()
    .max_connections(20)
    .min_connections(5)
    .acquire_timeout(Duration::from_secs(30))
    .connect(&database_url)
    .await?;

// Query optimization
sqlx::query_as!(
    User,
    r#"
    SELECT id, email, created_at
    FROM users 
    WHERE tenant_id = $1 
    AND active = true
    ORDER BY created_at DESC
    LIMIT $2
    "#,
    tenant_id,
    limit
)
.fetch_all(&pool)
.await?;
```

### Frontend Optimization

```typescript
// Lazy loading
export const LazyDashboard = lazy$(() => import('./dashboard'));

// Resource optimization
export const optimizedImage$ = $(async (src: string) => {
  const img = new Image();
  img.src = src;
  await new Promise((resolve) => {
    img.onload = resolve;
  });
  return img;
});
```

## Troubleshooting

### Common Issues

1. **Compilation Errors**
   ```bash
   # Clear Rust cache
   cargo clean
   
   # Update dependencies
   cargo update
   
   # Check for conflicts
   cargo tree --duplicates
   ```

2. **Database Connection Issues**
   ```bash
   # Check if PostgreSQL is running
   docker-compose ps postgres-dev
   
   # Test connection
   psql $DATABASE_URL -c "SELECT 1;"
   
   # Reset database
   docker-compose down -v
   docker-compose up -d
   ```

3. **Frontend Build Issues**
   ```bash
   # Clear node modules
   rm -rf node_modules package-lock.json
   pnpm install
   
   # Clear Qwik cache
   rm -rf .qwik dist
   pnpm build
   ```

### Debug Commands

```bash
# Check all services
docker-compose ps

# View logs
docker-compose logs -f postgres-dev
cargo run 2>&1 | grep ERROR

# Monitor resources
htop
docker stats

# Network debugging
netstat -tlnp | grep :3000
curl -v http://localhost:3000/health
```

This development guide provides everything needed to set up and work with QuillSpace locally. The modular architecture and comprehensive tooling ensure a productive development experience.
